// ============================================================================
// 配置：建好 GitHub 仓库后，把下面两个常量改成你的 owner / repo 名即可。
// 例如仓库 https://github.com/jtqjlow-bit/invoice-generator
//   GITHUB_OWNER = "jtqjlow-bit"
//   GITHUB_REPO  = "invoice-generator"
// 在没配置（仍是 YOUR_ 开头）之前，检查更新会静默跳过，不影响使用。
// ============================================================================
const GITHUB_OWNER = "jtqjlow-bit";
const GITHUB_REPO = "Selfuse-invoice-generator";

export interface UpdateInfo {
  /** Latest version without the leading "v", e.g. "0.2.0". */
  latestVersion: string;
  /** GitHub release page URL. */
  url: string;
  /** Direct download URL of the first .exe asset, if any. */
  downloadUrl: string | null;
}

/** Compare two semver-ish strings; returns true when `latest` is newer than `current`. */
export function isNewer(latest: string, current: string): boolean {
  const parse = (v: string) =>
    v
      .replace(/^v/i, "")
      .split(".")
      .map((n) => parseInt(n, 10) || 0);
  const a = parse(latest);
  const b = parse(current);
  const len = Math.max(a.length, b.length);
  for (let i = 0; i < len; i++) {
    const x = a[i] ?? 0;
    const y = b[i] ?? 0;
    if (x !== y) return x > y;
  }
  return false;
}

/**
 * Check GitHub for a newer release. Returns null when up to date, not
 * configured, offline, or on any error (update checks must never break the app).
 */
export async function checkForUpdate(
  currentVersion: string,
): Promise<UpdateInfo | null> {
  if (!currentVersion || GITHUB_OWNER.startsWith("YOUR_")) return null;
  try {
    const res = await fetch(
      `https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPO}/releases/latest`,
      { headers: { Accept: "application/vnd.github+json" } },
    );
    if (!res.ok) return null;
    const data: {
      tag_name?: string;
      html_url?: string;
      assets?: { name?: string; browser_download_url?: string }[];
    } = await res.json();

    const tag = data.tag_name ?? "";
    if (!tag || !isNewer(tag, currentVersion)) return null;

    const exe = (data.assets ?? []).find((a) =>
      (a.name ?? "").toLowerCase().endsWith(".exe"),
    );
    return {
      latestVersion: tag.replace(/^v/i, ""),
      url:
        data.html_url ??
        `https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/releases/latest`,
      downloadUrl: exe?.browser_download_url ?? null,
    };
  } catch {
    return null;
  }
}
