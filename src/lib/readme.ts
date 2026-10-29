// README cache for the session: switching tabs must not refetch. Keyed by
// tool and revision, so installing a new version shows its README.

import { getReadme } from "./api";
import type { ReadmeDto } from "./bindings/ReadmeDto";

const cache = new Map<string, Promise<ReadmeDto>>();

export function loadReadme(id: string, revision: string | null, force = false): Promise<ReadmeDto> {
  const key = `${id}@${revision ?? "-"}`;
  if (force) cache.delete(key);
  let pending = cache.get(key);
  if (!pending) {
    pending = getReadme(id).catch((e: Error) => {
      cache.delete(key);
      throw e;
    });
    cache.set(key, pending);
  }
  return pending;
}
