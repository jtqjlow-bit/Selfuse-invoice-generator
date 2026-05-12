# Invoice System — Codebase Guide

本仓库实现一款 Windows 本地运行的开单软件（Quotation / Invoice / Payment Voucher）。
技术栈：**Tauri 2 + React 18 + TypeScript + Rust + SQLite (rusqlite)**。
本文档定义代码组织规则。**所有修改必须遵守本文规则，否则模块间会发生数据污染。**

---

## 0. 工作准则（Claude Code 必须遵守）

1. **遇到歧义立即提问，不要猜测后继续执行**。需求不清楚就停下来问；任何"我假设你是 X 意思"都不允许 — 必须先问清楚再写。
2. **写最少的代码解决问题，不加未要求的功能**。不做未来扩展、不预留 hook、不引入未要求的抽象。YAGNI 原则严格执行。
3. **每次只改被要求修改的地方，不动无关代码**。看到别处有"顺便可以优化的"也不要动；要改的话先提出来等确认。
4. **完成后说明如何验证结果**。每次改完必须说："这个改动可以这样验证：...（具体步骤）"。验证方式包括：跑哪个测试、点哪个按钮、查哪张表、看哪个文件。

---

## 1. 架构原则（不可违反）

1. **严格分层**：UI (TS) → Tauri Command Bridge → Service (Rust) → Domain (Rust) → Infra (Rust)。**禁止反向依赖**。
2. **单一数据所有权**：每张 SQLite 表只属于一个 Rust Domain 模块（其 `repository.rs` 是唯一可执行 SQL 的地方）。其他模块**只能通过该模块 `commands.rs` 暴露的 Tauri 命令或 `mod.rs` 公开函数**访问，**禁止跨模块 SQL 查询**。
3. **跨模块通信只通过函数调用 / 数据快照**。禁止跨模块直接读对方 ORM struct 字段去做计算 — 拿快照（owned 值）。
4. **Service 层无状态**，或仅持有自己缓存表（如 `service::currency` 的汇率缓存）。Service 不持有 Domain 数据。
5. **前端永远不直接访问 SQLite**。前端只通过 `@tauri-apps/api/core` 的 `invoke()` 调用 Rust commands。
6. **每个 Rust 模块一个文件夹**，对外接口只通过 `mod.rs` 与 `commands.rs` 暴露。
7. **每个 React feature 一个文件夹**（`src/features/<name>/`），对外接口只通过 `index.ts` 暴露。
8. **禁止 god-module**。Rust 单文件超 400 行或 React 组件超 300 行，强制拆分。
9. **禁止循环依赖**。任何 PR 加入循环依赖时拒绝合并。

---

## 2. 目录结构

```
invoice-system/
├── CLAUDE.md
├── README.md
├── package.json                    # React + Vite 依赖
├── pnpm-lock.yaml (or package-lock.json)
├── vite.config.ts
├── tsconfig.json
├── index.html                      # Vite 入口
│
├── src/                            # React 前端（TypeScript）
│   ├── main.tsx                    # 入口
│   ├── App.tsx                     # 根路由
│   ├── api/                        # Tauri invoke 封装（与 Rust commands 一一对应）
│   │   ├── customer.ts
│   │   ├── quotation.ts
│   │   ├── invoice.ts
│   │   ├── payment_voucher.ts
│   │   ├── company_settings.ts
│   │   ├── pdf_template.ts
│   │   ├── currency.ts
│   │   ├── numbering.ts
│   │   ├── pdf_renderer.ts
│   │   ├── backup_restore.ts
│   │   ├── import_export.ts
│   │   ├── report.ts
│   │   └── dashboard.ts
│   ├── types/                      # ts-rs 从 Rust 自动生成的类型
│   │   └── bindings/
│   ├── features/                   # 按 domain 组织的 UI 功能
│   │   ├── dashboard/
│   │   │   ├── index.ts            # 唯一对外接口
│   │   │   ├── DashboardPage.tsx
│   │   │   └── ...
│   │   ├── customer/
│   │   ├── quotation/
│   │   ├── invoice/
│   │   ├── payment_voucher/
│   │   ├── settings/
│   │   ├── report/
│   │   └── backup/
│   ├── common/                     # 共享 UI 组件
│   │   ├── components/             # Button, Modal, Table, Toast, FormField
│   │   ├── hooks/
│   │   └── utils/
│   └── i18n/                       # 中文字符串表
│       └── zh.json
│
├── src-tauri/                      # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json             # Tauri 配置（含 fs scope 等权限）
│   ├── build.rs
│   ├── icons/
│   ├── migrations/                 # SQLite 迁移 SQL
│   │   ├── 0001_initial.sql
│   │   └── ...
│   └── src/
│       ├── main.rs                 # Tauri 启动
│       ├── lib.rs                  # 命令注册（invoke_handler!）
│       ├── error.rs                # 统一错误类型 AppError
│       │
│       ├── infra/                  # 基础设施层
│       │   ├── mod.rs
│       │   ├── db.rs               # rusqlite Pool + 迁移
│       │   ├── config.rs           # 用户配置读写
│       │   └── file_system.rs      # 文件 IO 抽象
│       │
│       ├── domain/                 # 领域层（独占 DB 表）
│       │   ├── mod.rs
│       │   ├── customer/
│       │   │   ├── mod.rs          # 公开 API（pub fn）
│       │   │   ├── types.rs        # serde + ts-rs 派生的 struct
│       │   │   ├── repository.rs   # SQL 访问（唯一允许写 SQL 的地方）
│       │   │   ├── service.rs      # 业务逻辑
│       │   │   ├── commands.rs     # #[tauri::command] 包装
│       │   │   └── tests.rs
│       │   ├── quotation/
│       │   │   ├── mod.rs
│       │   │   ├── types.rs
│       │   │   ├── repository.rs
│       │   │   ├── service.rs
│       │   │   ├── state_machine.rs
│       │   │   ├── commands.rs
│       │   │   └── tests.rs
│       │   ├── invoice/
│       │   ├── payment_voucher/
│       │   ├── company_settings/
│       │   └── pdf_template/
│       │
│       └── service/                # 跨域服务层（无 DB 表或独立缓存表）
│           ├── mod.rs
│           ├── numbering/
│           ├── currency/
│           ├── tax_calc/
│           ├── pdf_renderer/
│           ├── backup_restore/
│           ├── import_export/
│           ├── report/
│           └── dashboard/
│
└── tests/                          # 跨模块集成测试（Rust）
    └── integration/
```

每个 Rust 模块内部固定结构：
- `mod.rs` — 模块入口，`pub use` 公开 API
- `types.rs` — 类型定义（含 `#[derive(Serialize, Deserialize, TS)]`）
- `repository.rs` — DB 访问（**仅 Domain 层有**）
- `service.rs` — 业务逻辑
- `state_machine.rs` — 状态机（仅有状态流转的模块有）
- `commands.rs` — `#[tauri::command]` 函数（公开给前端的接口）
- `tests.rs` — 单元测试

每个 React feature 内部结构：
- `index.ts` — 唯一对外接口，`export` 该 feature 的 page / hooks
- `<Feature>Page.tsx` — 主页面组件
- `components/` — 该 feature 私有组件
- `hooks/` — 该 feature 私有 hooks

---

## 3. Tauri 命令约定（前后端通信契约）

### 3.1 命令命名

格式：`<module>_<action>`，全部小写蛇形：
- `customer_create`、`customer_update`、`customer_find_by_id`、`customer_list`
- `quotation_mark_accepted`、`quotation_convert_to_invoice`
- `invoice_mark_paid`、`invoice_recalc_paid_amount`
- `pdf_render_invoice`、`backup_export_zip`

### 3.2 命令签名

```rust
// src-tauri/src/domain/customer/commands.rs
#[tauri::command]
pub async fn customer_create(
    app: tauri::AppHandle,
    payload: CreateCustomerInput,
) -> Result<Customer, AppError> {
    let state = app.state::<AppState>();
    crate::domain::customer::service::create(&state.db, payload).await
}
```

**规则**：
- 入参用单一 `payload` struct（不要散落参数）
- 出参用 owned 类型（避免生命周期问题）
- 错误统一返回 `AppError`（实现 `Serialize`）

### 3.3 前端调用约定

```ts
// src/api/customer.ts
import { invoke } from '@tauri-apps/api/core';
import type { Customer, CreateCustomerInput } from '@/types/bindings';

export const customerApi = {
  create: (payload: CreateCustomerInput) =>
    invoke<Customer>('customer_create', { payload }),
  list: () => invoke<Customer[]>('customer_list'),
  // ...
};
```

**规则**：
- 前端只通过 `src/api/<module>.ts` 调 invoke，不在组件里散写 `invoke('xxx')`
- 类型通过 `ts-rs` 从 Rust 自动生成到 `src/types/bindings/`

### 3.4 命令注册

所有 `#[tauri::command]` 在 `lib.rs` 的 `invoke_handler!` 集中注册：

```rust
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        domain::customer::commands::customer_create,
        domain::customer::commands::customer_list,
        // ...
    ])
    .run(...)
```

---

## 4. 数据所有权矩阵

**每张 DB 表只能由对应模块的 `repository.rs` 直接读写。** 其他模块要访问该表的数据，必须调用该模块的 `mod.rs` 暴露的 Rust 函数或 `commands.rs` 暴露的 Tauri 命令。

| DB 表 | 所有者模块 | 其他模块如何访问 |
|---|---|---|
| `customer` | `domain::customer` | `domain::customer::find_by_id()` / `list()` 等 |
| `quotation` | `domain::quotation` | `domain::quotation::*` |
| `quotation_line_item` | `domain::quotation` | 内部表，不对外 |
| `invoice` | `domain::invoice` | `domain::invoice::*` |
| `invoice_line_item` | `domain::invoice` | 内部表，不对外 |
| `payment_voucher` | `domain::payment_voucher` | `domain::payment_voucher::*` |
| `company_settings` | `domain::company_settings` | `domain::company_settings::get()` / `update()` |
| `pdf_template` | `domain::pdf_template` | `domain::pdf_template::*` |
| `numbering_counter` | `service::numbering` | `service::numbering::next()` / `peek()` 等 |
| `exchange_rate_cache` | `service::currency` | `service::currency::get_rate()` / `convert()` |

**禁止行为示例**：
```rust
// 错误：dashboard 直接查 invoice 表
let invoices: Vec<Invoice> = conn.prepare("SELECT * FROM invoice ...")?.query_map(...);

// 正确：通过 invoice 模块公开 API
use crate::domain::invoice;
let recent = invoice::list_recent(&db, 5).await?;
```

```ts
// 错误：组件里直接 invoke
const customers = await invoke('customer_list');

// 正确：通过 api 层
import { customerApi } from '@/api/customer';
const customers = await customerApi.list();
```

---

## 5. 模块依赖规则

```
src/features/*          → 可依赖：src/api/*, src/common/*, src/types/*
src/api/*               → 只调 invoke()，不依赖业务模块
src-tauri service/*     → 可依赖：domain/*, infra/*
src-tauri domain/*      → 可依赖：infra/* （同层依赖见下文白名单）
src-tauri infra/*       → 不依赖任何业务模块
```

**Domain 层同层依赖白名单（仅此 3 条）**：
- `domain::quotation` 可依赖 `domain::customer`（读客户做快照）
- `domain::invoice` 可依赖 `domain::customer` 和 `domain::quotation`（从 Quotation 创建 Invoice）
- `domain::payment_voucher` 可依赖 `domain::customer` 和 `domain::invoice`（创建后回调 invoice 重算）

**反向严禁**：
- `domain::customer` 不能依赖 `quotation` / `invoice` / `payment_voucher`
- `service` 不能依赖 `tauri::command` 命令（service 是 Rust 内部，不通过 invoke 跳来跳去）
- 前端 `features` 不能 import 其他 `features` 的内部文件，只能用对方 `index.ts` 公开的东西

---

## 6. 各模块详细规格

### 6.1 `infra::db`

- 职责：rusqlite 连接封装、迁移、事务封装（单进程单用户，单连接 + `Mutex` 即可，不上连接池）
- 公开：
  - `pub struct Db` — 持有 `Mutex<Connection>` 与 db path
  - `pub fn open(path: impl AsRef<Path>) -> AppResult<Self>` — 自动建父目录，启用 WAL + foreign_keys + synchronous=NORMAL
  - `pub fn run_migrations(&self) -> AppResult<()>` — 自维护 `_migrations` 表，逐文件单事务应用
  - `pub fn with_conn<F, R>(&self, f: F) -> AppResult<R>` — 读用，传入连接引用
  - `pub fn transaction<F, R>(&self, f: F) -> AppResult<R>` — 写用，闭包返回 `Err` 自动回滚
- 接口同步：rusqlite 本身同步，Tauri 命令侧如需避免阻塞 runtime 可用 `tauri::async_runtime::spawn_blocking` 包一层
- **禁止**：业务模块直接拿 `Db` 写 raw SQL；必须通过自己的 `repository.rs`

### 6.2 `infra::config`

- 职责：读写应用配置
- 存储：`%APPDATA%\InvoiceSystem\config.json`
- 公开：`get(key) / set(key, value) / data_dir() / set_data_dir()`

### 6.3 `infra::file_system`

- 职责：受 Tauri scope 限制的文件 IO
- 公开：`read_file / write_file / ensure_dir / zip_dir / unzip / copy / delete`
- **禁止**：业务模块直接用 `std::fs` 或 `tokio::fs`；都走这层

### 6.4 `domain::customer`

- **拥有的表**：`customer`
- 字段：
  - `id: String` (UUID, PK)
  - `type_: CustomerType` (`Company | Individual`)
  - `name: String`
  - `contact_person: Option<String>`
  - `email: Option<String>`
  - `phone: Option<String>`
  - `address: Option<String>`
  - `ssm_no: Option<String>` (company 必填，service 层校验)
  - `nric: Option<String>` (individual 必填，service 层校验)
  - `tax_no: Option<String>` (SST 号等)
  - `notes: Option<String>`
  - `archived: bool`
  - `created_at: DateTime<Utc>`, `updated_at: DateTime<Utc>`
- 公开函数：`create / update / find_by_id / list / archive / unarchive / search`
- 公开 Tauri 命令：对应同名 `customer_*`
- **不依赖任何其他 domain**

### 6.5 `domain::quotation`

- **拥有的表**：`quotation`, `quotation_line_item`
- `quotation` 关键字段：
  - `id, number` (e.g. `QUO-2026-001`, unique)
  - `customer_id`（FK）
  - `customer_snapshot: serde_json::Value`（开单时客户信息快照，用于 PDF 不变）
  - `issue_date, valid_until` (默认 +30 天)
  - `currency` (e.g. `MYR`)
  - `tax_enabled: bool, tax_rate: Option<f64>`
  - `subtotal, tax_amount, total: f64`
  - `status: QuotationStatus`（`Draft / Sent / Accepted / Rejected / Expired`）
  - `converted_invoice_id: Option<String>`
  - `notes, terms: Option<String>`
- `quotation_line_item` 字段：
  - `id, quotation_id, position`
  - `description: String` (支持换行，UI 用 `<textarea>`)
  - `quantity, unit_price, line_total: f64`
  - `line_currency: String` (默认同单据)
  - `exchange_rate_to_doc_currency: Option<f64>` (当 line_currency != quotation.currency)
  - `tax_rate: Option<f64>` (line 级覆盖)
  - `discount_rate: Option<f64>`
- 状态机（见 `state_machine.rs`）：`Draft → Sent → (Accepted | Rejected | Expired)`
- 公开函数：`create / update / mark_sent / mark_accepted / mark_rejected / find_by_id / list / list_by_customer / build_snapshot_for_invoice`
- 依赖：`domain::customer`、`service::numbering`、`service::currency`、`service::tax_calc`

### 6.6 `domain::invoice`

- **拥有的表**：`invoice`, `invoice_line_item`
- 与 `quotation` 类似，加：
  - `due_date: NaiveDate`
  - `source_quotation_id: Option<String>`
  - `payment_methods_snapshot: serde_json::Value`
  - `status: InvoiceStatus`（`Draft / Sent / PartialPaid / Paid / Overdue / Void`）
  - `paid_amount: f64`（PV 合计自动更新）
- 状态机：
  ```
  Draft → Sent → (PartialPaid → Paid)
                    ↓
                  Overdue (自动；可人工取消回 Sent / PartialPaid)
                    ↓
                  Void (手动)
  ```
- 公开函数：`create / create_from_quotation_snapshot / update / mark_sent / mark_paid / mark_partial_paid / mark_void / cancel_overdue / find_by_id / list / list_by_customer / recalc_paid_amount / auto_mark_overdue_all`
- 依赖：`domain::customer`、`domain::quotation`、`service::numbering`、`service::currency`、`service::tax_calc`
- **被依赖**：`domain::payment_voucher` 调 `recalc_paid_amount`

### 6.7 `domain::payment_voucher`

- **拥有的表**：`payment_voucher`
- 字段：
  - `id, number` (`PV-2026-001`)
  - `invoice_id`（FK）
  - `customer_id`（冗余便于查询）
  - `customer_snapshot: serde_json::Value`
  - `date: NaiveDate`
  - `amount: f64, currency: String`
  - `payment_method: String`
  - `notes: Option<String>`
  - `created_at`
- 创建/删除后必须调 `invoice::recalc_paid_amount(invoice_id)`
- 公开函数：`create / update / delete / find_by_id / list / list_by_invoice / list_by_customer / sum_by_invoice`
- 依赖：`domain::customer`、`domain::invoice`

### 6.8 `domain::company_settings`

- **拥有的表**：`company_settings`（**单条记录，id 永远 1**）
- 字段：
  - 公司名 / 地址 / Email / 电话 / SSM 号 / SST 号
  - `logo_path: Option<String>`
  - `qr_path: Option<String>` (FPX / DuitNow 固定 QR)
  - `bank_accounts: serde_json::Value`
  - `enabled_payment_methods: serde_json::Value`
  - `default_tax_rate: Option<f64>`
  - `default_quotation_valid_days: i32` (默认 30)
  - `default_invoice_due_days: i32`
  - `data_dir: String` (PDF 输出目录等)
- 公开函数：`get() / update()`
- 依赖：无

### 6.9 `domain::pdf_template`

- **拥有的表**：`pdf_template`
- 字段：
  - `id, name, type_` (`Preset | Custom`)
  - `file_path: String`
  - `config_json: serde_json::Value` (主色、字体等可调项)
- 预设模板 3 套，启动时检测并填入 `pdf_template` 表
- 用户上传：HTML 文件 + Tera 占位符（见 §8）
- 公开函数：`list / find_by_id / upload_custom / delete_custom / get_renderable(id)`
- 依赖：`infra::file_system`

### 6.10 `service::numbering`

- **拥有的表**：`numbering_counter` (`doc_type, year, last_seq`)
- 算法：`{prefix}-{year}-{seq:03}`，年度重置
- 公开函数：
  - `next(db, DocType) -> Result<String>` — 事务内原子递增
  - `peek(db, DocType) -> Result<String>` — 不递增，预览
  - `set_override(db, DocType, year, seq) -> Result<()>` — 用户手动改号后回写
- 依赖：`infra::db`
- 单机 SQLite 事务保证并发安全

### 6.11 `service::currency`

- **拥有的表**：`exchange_rate_cache`
- 字段：`base, target, rate, fetched_at`
- 公开函数：
  - `get_rate(from, to) -> Result<f64>` — 24h 缓存，过期联网拉
  - `convert(amount, from, to) -> Result<f64>`
  - `refresh() -> Result<()>` — 强制刷新所有缓存
- 依赖：`infra::db`，外部 HTTP API（具体选型见 §14 TBD）

### 6.12 `service::tax_calc`

- **无 DB 表**（纯函数）
- 公开函数：
  - `line_tax(amount, tax_rate, tax_inclusive=false) -> (tax, total)`
  - `document_totals(line_items, tax_enabled, default_tax_rate) -> Totals`
- 依赖：无

### 6.13 `service::pdf_renderer`

- **无 DB 表**
- 公开函数：
  - `render_quotation(quotation_dto, template_id) -> Result<Vec<u8>>` 或写文件路径
  - `render_invoice(invoice_dto, template_id) -> Result<...>`
  - `render_payment_voucher(pv_dto, template_id) -> Result<...>`
- 输入只接 DTO（已组装好的快照），**不查 DB**
- 依赖：`domain::pdf_template`（拿模板）、`infra::file_system`
- 实现：Tera 模板引擎填充 HTML → `headless_chrome` crate（驱动随包打包的 Chromium）→ PDF
- 打包：Chromium 通过 `tauri.conf.json` 的 resources 字段或 build script 嵌入安装包；启动时定位到 resource 目录

### 6.14 `service::backup_restore`

- **无 DB 表**
- 公开函数：
  - `export_zip(target_path) -> Result<()>` — 打包 SQLite 文件 + PDF 输出目录 + Logo + QR + 自定义模板
  - `restore_zip(zip_path) -> Result<()>` — 操作前自动在临时目录备份当前数据，再覆盖
- 依赖：`infra::file_system`、`infra::db`、`infra::config`

### 6.15 `service::import_export`

- **无 DB 表**
- 公开函数：
  - `import_customers_from_csv(file_path) -> Result<ImportReport>`
  - `export_all_to_excel(target_path) -> Result<()>` (多 sheet：客户 / 报价 / 发票 / 收款凭证)
- 依赖：`domain::customer`、`domain::quotation`、`domain::invoice`、`domain::payment_voucher`
- **只调对方公开函数，不查表**

### 6.16 `service::report`

- **无 DB 表**
- 公开函数：
  - `monthly_revenue(year, month) -> MonthlyReport`
  - `yearly_revenue(year) -> YearlyReport`
  - `outstanding_invoices() -> Vec<Invoice>`
- 依赖：`domain::invoice`、`domain::payment_voucher`、`domain::customer`
- **禁止**：直接 SQL 查表。若 domain 没有合适聚合 API，先去 domain 加。

### 6.17 `service::dashboard`

- **无 DB 表**
- 公开函数：
  - `get_dashboard_data() -> DashboardData`（本月营收 / 未付款总额 / 最近 5 张单）
- 依赖：`service::report`、`domain::invoice`、`domain::quotation`、`domain::payment_voucher`

### 6.18 React `features/*`

- 每个 feature 对应一个 domain（`features/customer`、`features/quotation` 等）
- 共享组件放 `src/common/components/`
- 数据访问**只**通过 `src/api/<module>.ts` 调 Tauri 命令
- **禁止**：`features/customer` import `features/invoice` 的内部组件；如有跨 feature 需求，把组件升到 `common/`

---

## 7. 状态机文件约定

任何有状态流转的 Domain 模块（`quotation`、`invoice`）必须有独立 `state_machine.rs`：

```rust
// domain/invoice/state_machine.rs
use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub enum InvoiceStatus {
    Draft,
    Sent,
    PartialPaid,
    Paid,
    Overdue,
    Void,
}

pub fn can_transition(from: InvoiceStatus, to: InvoiceStatus) -> bool {
    use InvoiceStatus::*;
    matches!(
        (from, to),
        (Draft, Sent)
            | (Sent, PartialPaid)
            | (Sent, Paid)
            | (Sent, Overdue)
            | (Sent, Void)
            | (PartialPaid, Paid)
            | (PartialPaid, Overdue)
            | (PartialPaid, Void)
            | (Overdue, Sent)
            | (Overdue, PartialPaid)
            | (Overdue, Paid)
            | (Overdue, Void)
            | (Paid, Void)
    )
}

pub fn transition(current: InvoiceStatus, to: InvoiceStatus) -> Result<InvoiceStatus, AppError> {
    if !can_transition(current, to) {
        return Err(AppError::InvalidTransition { from: current, to });
    }
    Ok(to)
}
```

**修改状态只能通过 `transition()`**，不允许直接 `invoice.status = X`。

---

## 8. PDF 模板格式约定（自定义模板用）

为了允许用户上传自定义模板又保持简单，约定：
- 模板是 **HTML 文件**，使用 **Tera 语法**（`{{ var }}` / `{% for %}` / `{% if %}` / filters）
- 渲染：Tera 填充数据 → 输出 HTML → `headless_chrome` 转 PDF
- 完整占位符列表与示例必须维护于 `docs/pdf_placeholders.md`，新增字段时更新
- 内置自定义 Tera filter：`currency(code=...)`、`date(format=...)`、`nl2br`

---

## 9. 命名约定

| 范畴 | 约定 | 示例 |
|---|---|---|
| Rust 文件 / 文件夹 | `snake_case` | `payment_voucher/` |
| Rust 函数 / 变量 | `snake_case` | `find_by_id` |
| Rust struct / enum | `PascalCase` | `InvoiceStatus` |
| Rust 常量 | `UPPER_SNAKE` | `DEFAULT_TAX_RATE` |
| Rust 模块文件 | `mod.rs / types.rs / repository.rs / service.rs / commands.rs / state_machine.rs` |
| TS 文件夹 | `kebab-case` 或 `snake_case`（项目内统一） | `features/payment-voucher/` |
| TS 组件 | `PascalCase.tsx` | `InvoiceForm.tsx` |
| TS hooks | `useXxx.ts` | `useInvoiceList.ts` |
| TS 普通函数 / 变量 | `camelCase` | `formatCurrency` |
| TS 类型 | `PascalCase` | `Invoice` |
| DB 表 / 列 | `snake_case` | `invoice_line_item` |
| Tauri 命令 | `snake_case`，`<module>_<action>` | `customer_create` |

---

## 10. 错误处理

- 后端统一错误类型 `AppError`（在 `src-tauri/src/error.rs`），实现 `Serialize`
- 所有 `#[tauri::command]` 返回 `Result<T, AppError>`
- 前端 `invoke()` 失败时拿到 `AppError` 的 JSON 表示，用 toast 显示给用户
- 业务错误（如客户不存在）和系统错误（如 DB 连接失败）都走 `AppError`，但有不同 variant 便于 UI 区分

---

## 11. 测试要求

- **每个 Domain 模块** 必须有 `tests.rs`，覆盖：CRUD + 状态机所有合法/非法转换
- **每个 Service 模块** 必须有 `tests.rs`，覆盖核心算法
- **跨模块流程** 用 `tests/integration/` 测试（如：创建 Quotation → 接受 → 转 Invoice → 标记付款 → 生成 PV）
- 测试用临时 SQLite 文件（`tempfile` crate），每个测试独立
- 前端组件测试用 Vitest + React Testing Library

---

## 12. 禁止行为清单（违反必拒）

1. 跨模块直接 SQL 查询（必须通过对方 `mod.rs` 公开函数）
2. Domain 层反向 import UI / 调 Tauri command
3. `domain::customer` import `quotation` / `invoice` / `payment_voucher`
4. 状态字段上直接赋值（必须走 `transition()` 函数）
5. 业务模块直接用 `std::fs` / `tokio::fs`（走 `infra::file_system`）
6. 任何模块 import 自己 `mod.rs` / `index.ts` 没暴露的内部文件
7. 编号生成跨过 `service::numbering` 自己拼
8. `service::report` / `service::dashboard` 跨过 domain 直接读表
9. 前端组件里散写 `invoke('xxx')` — 必须走 `src/api/<module>.ts`
10. 删除单据时硬删（必须走 `mark_void` 或软删除）
11. 前端 import 其他 feature 的内部文件
12. Rust 代码 `unwrap()` Result（除了测试和 main 早期）— 用 `?` 传播

---

## 13. 加一个新功能时该怎么做

1. 判断功能属于 **哪一层**（domain 加表？service 跨域聚合？UI 加界面？）
2. 列出**依赖的模块**，验证不违反层级规则
3. 后端：
   - 在对应 domain/service 模块的 `service.rs` 写业务逻辑
   - 在 `commands.rs` 加 Tauri 命令
   - 在 `lib.rs` 的 `invoke_handler!` 注册
   - 在 `tests.rs` 加测试
4. 前端：
   - 在 `src/api/<module>.ts` 加 invoke 包装
   - 在 `src/types/bindings/` 等 `ts-rs` 自动生成新类型
   - 在 `src/features/<module>/` 加 UI
5. **不要**为了"以后用得上"加任何当前不需要的参数 / 字段
6. 更新本文档对应章节

---

## 14. 已定栈 + 待决定

**已定栈**：
- 前端框架：React 18 + TypeScript + Vite
- 后端语言：Rust（Tauri 2）
- 数据库：SQLite（rusqlite）
- 类型桥接：ts-rs
- **UI 库：shadcn/ui + Tailwind CSS**
- **PDF 引擎：headless_chrome（HTML → PDF）+ 随包打包 Chromium**
- **HTML 模板引擎：Tera**

**仍需开发协同补完（开发期间确定）**：
- 汇率 API：fixer.io / exchangerate.host / 其他免费方案
- Excel 库：建议 calamine（读）+ rust_xlsxwriter（写）
- 自定义模板边界：是否允许上传非 HTML 格式
- 备份 zip 是否加密
- 前端表单库：建议 react-hook-form
- 路由库：建议 React Router
