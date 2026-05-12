import { invoke } from "@tauri-apps/api/core";
import type { BusinessProfile } from "@/types/bindings/BusinessProfile";
import type { CreateBusinessProfileInput } from "@/types/bindings/CreateBusinessProfileInput";
import type { ProfileAssetDataUrls } from "@/types/bindings/ProfileAssetDataUrls";
import type { QrKind } from "@/types/bindings/QrKind";
import type { UpdateBusinessProfileInput } from "@/types/bindings/UpdateBusinessProfileInput";

export const businessProfileApi = {
  list: () => invoke<BusinessProfile[]>("business_profile_list"),
  findById: (id: string) =>
    invoke<BusinessProfile>("business_profile_find_by_id", { id }),
  create: (payload: CreateBusinessProfileInput) =>
    invoke<BusinessProfile>("business_profile_create", { payload }),
  update: (payload: UpdateBusinessProfileInput) =>
    invoke<BusinessProfile>("business_profile_update", { payload }),
  delete: (id: string) =>
    invoke<void>("business_profile_delete", { id }),
  setLogo: (id: string, bytesB64: string, ext: string) =>
    invoke<BusinessProfile>("business_profile_set_logo", {
      id,
      bytesB64,
      ext,
    }),
  clearLogo: (id: string) =>
    invoke<BusinessProfile>("business_profile_clear_logo", { id }),
  setQr: (id: string, bytesB64: string, ext: string) =>
    invoke<BusinessProfile>("business_profile_set_qr", { id, bytesB64, ext }),
  clearQr: (id: string) =>
    invoke<BusinessProfile>("business_profile_clear_qr", { id }),
  addQr: (
    id: string,
    kind: QrKind,
    label: string,
    bytesB64: string,
    ext: string,
  ) =>
    invoke<BusinessProfile>("business_profile_add_qr", {
      id,
      kind,
      label,
      bytesB64,
      ext,
    }),
  removeQr: (id: string, qrId: string) =>
    invoke<BusinessProfile>("business_profile_remove_qr", { id, qrId }),
  updateQrLabel: (id: string, qrId: string, label: string) =>
    invoke<BusinessProfile>("business_profile_update_qr_label", {
      id,
      qrId,
      label,
    }),
  getAssetDataUrls: (id: string) =>
    invoke<ProfileAssetDataUrls>("business_profile_get_asset_data_urls", { id }),
};

/** Soft upload limits. The backend enforces these too, but we check up-front
 *  to give a better error before paying the base64 + IPC cost. */
export const MAX_LOGO_BYTES = 2 * 1024 * 1024;
export const MAX_QR_BYTES = 1 * 1024 * 1024;

/** Read a File as base64 string (no `data:...;base64,` prefix). */
export async function fileToBase64(file: File): Promise<string> {
  const buf = await file.arrayBuffer();
  const bytes = new Uint8Array(buf);
  let bin = "";
  const CHUNK = 0x8000;
  for (let i = 0; i < bytes.length; i += CHUNK) {
    bin += String.fromCharCode(...bytes.subarray(i, i + CHUNK));
  }
  return btoa(bin);
}
