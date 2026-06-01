pub mod domain;
pub mod error;
pub mod infra;
pub mod service;

use std::path::PathBuf;

use tauri::Manager;

use crate::error::AppError;
use crate::infra::Db;

pub struct AppState {
    pub db: Db,
    pub data_dir: PathBuf,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| AppError::Internal(format!("app_data_dir resolve failed: {e}")))?;
            // Must run BEFORE Db::open — replacement is a swap of invoice.db
            // and SQLite holds an OS file lock once the connection is open.
            service::backup_restore::apply_pending_restore(&data_dir)?;
            let db = Db::open(data_dir.join("invoice.db"))?;
            db.run_migrations()?;
            let state = AppState { db, data_dir };
            // Auto-flip past-due Sent / PartialPaid invoices to Overdue at startup.
            let _ = domain::invoice::auto_mark_overdue_all(&state.db);
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            domain::business_profile::commands::business_profile_list,
            domain::business_profile::commands::business_profile_find_by_id,
            domain::business_profile::commands::business_profile_create,
            domain::business_profile::commands::business_profile_update,
            domain::business_profile::commands::business_profile_delete,
            domain::business_profile::commands::business_profile_set_logo,
            domain::business_profile::commands::business_profile_clear_logo,
            domain::business_profile::commands::business_profile_set_qr,
            domain::business_profile::commands::business_profile_clear_qr,
            domain::business_profile::commands::business_profile_add_qr,
            domain::business_profile::commands::business_profile_remove_qr,
            domain::business_profile::commands::business_profile_update_qr_label,
            domain::business_profile::commands::business_profile_get_asset_data_urls,
            domain::customer::commands::customer_create,
            domain::customer::commands::customer_update,
            domain::customer::commands::customer_find_by_id,
            domain::customer::commands::customer_list,
            domain::customer::commands::customer_search,
            domain::customer::commands::customer_archive,
            domain::customer::commands::customer_unarchive,
            service::numbering::commands::numbering_peek,
            service::numbering::commands::numbering_set_override,
            domain::quotation::commands::quotation_create,
            domain::quotation::commands::quotation_update,
            domain::quotation::commands::quotation_find_by_id,
            domain::quotation::commands::quotation_list,
            domain::quotation::commands::quotation_list_by_customer,
            domain::quotation::commands::quotation_mark_sent,
            domain::quotation::commands::quotation_mark_accepted,
            domain::quotation::commands::quotation_mark_rejected,
            domain::quotation::commands::quotation_mark_expired,
            domain::invoice::commands::invoice_create,
            domain::invoice::commands::invoice_create_from_quotation,
            domain::invoice::commands::invoice_update,
            domain::invoice::commands::invoice_find_by_id,
            domain::invoice::commands::invoice_list,
            domain::invoice::commands::invoice_list_by_customer,
            domain::invoice::commands::invoice_mark_sent,
            domain::invoice::commands::invoice_mark_partial_paid,
            domain::invoice::commands::invoice_mark_paid,
            domain::invoice::commands::invoice_mark_overdue,
            domain::invoice::commands::invoice_mark_void,
            domain::invoice::commands::invoice_cancel_overdue,
            domain::invoice::commands::invoice_restore_void,
            domain::payment_voucher::commands::payment_voucher_create,
            domain::payment_voucher::commands::payment_voucher_update,
            domain::payment_voucher::commands::payment_voucher_delete,
            domain::payment_voucher::commands::payment_voucher_find_by_id,
            domain::payment_voucher::commands::payment_voucher_list,
            domain::payment_voucher::commands::payment_voucher_list_by_invoice,
            domain::payment_voucher::commands::payment_voucher_list_by_customer,
            domain::payment_voucher::commands::payment_voucher_sum_by_invoice,
            domain::pdf_template::commands::pdf_template_list,
            domain::pdf_template::commands::pdf_template_list_by_doc_type,
            domain::pdf_template::commands::pdf_template_find_by_id,
            domain::pdf_template::commands::pdf_template_upload_custom,
            domain::pdf_template::commands::pdf_template_delete_custom,
            domain::pdf_template::commands::pdf_template_get_renderable,
            service::pdf_renderer::commands::pdf_render_quotation,
            service::pdf_renderer::commands::pdf_render_invoice,
            service::pdf_renderer::commands::pdf_render_payment_voucher,
            service::pdf_renderer::commands::pdf_preview_quotation_html,
            service::pdf_renderer::commands::pdf_preview_invoice_html,
            service::pdf_renderer::commands::pdf_preview_payment_voucher_html,
            service::pdf_renderer::commands::pdf_render_template_sample,
            service::backup_restore::commands::backup_export_zip,
            service::backup_restore::commands::backup_restore_zip,
            service::dashboard::commands::dashboard_get_data,
            service::report::commands::report_monthly_revenue,
            service::report::commands::report_yearly_revenue,
            service::report::commands::report_outstanding_invoices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
