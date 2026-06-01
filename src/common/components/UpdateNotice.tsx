import { useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { checkForUpdate, type UpdateInfo } from "@/common/utils/updateCheck";

export function UpdateNotice({ currentVersion }: { currentVersion: string }) {
  const [info, setInfo] = useState<UpdateInfo | null>(null);

  useEffect(() => {
    if (!currentVersion) return;
    checkForUpdate(currentVersion)
      .then(setInfo)
      .catch(() => setInfo(null));
  }, [currentVersion]);

  if (!info) return null;

  return (
    <div className="mt-2 rounded-md border border-blue-300 bg-blue-50 p-2">
      <div className="mb-1 font-medium text-blue-800">
        发现新版本 v{info.latestVersion}
      </div>
      <button
        type="button"
        onClick={() => openUrl(info.downloadUrl ?? info.url)}
        className="rounded bg-blue-600 px-2 py-1 text-white hover:bg-blue-700"
      >
        下载更新
      </button>
    </div>
  );
}
