import { invoke } from "@tauri-apps/api/core";
import type { Customer } from "@/types/bindings/Customer";
import type { CreateCustomerInput } from "@/types/bindings/CreateCustomerInput";
import type { UpdateCustomerInput } from "@/types/bindings/UpdateCustomerInput";

export const customerApi = {
  create: (payload: CreateCustomerInput) =>
    invoke<Customer>("customer_create", { payload }),
  update: (payload: UpdateCustomerInput) =>
    invoke<Customer>("customer_update", { payload }),
  findById: (id: string) => invoke<Customer>("customer_find_by_id", { id }),
  list: (includeArchived: boolean) =>
    invoke<Customer[]>("customer_list", { includeArchived }),
  search: (query: string, includeArchived: boolean) =>
    invoke<Customer[]>("customer_search", { query, includeArchived }),
  archive: (id: string) => invoke<Customer>("customer_archive", { id }),
  unarchive: (id: string) => invoke<Customer>("customer_unarchive", { id }),
};
