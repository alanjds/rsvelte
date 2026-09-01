import { createHash } from "node:crypto";
import { OPEN_PHASE } from "./edits.mjs";

const digest = (value) =>
  createHash("sha256").update(JSON.stringify(value)).digest("hex").slice(0, 16);

export function compactCorpusObservation(
  method,
  position,
  differences,
  mechanisms = ["unclassified"],
) {
  return {
    method,
    position,
    diffDigest: digest([...differences].sort()),
    fieldCount: differences.length,
    mechanisms,
  };
}

export function aggregateCorpusDifferences(
  fileId,
  observations,
  phase = OPEN_PHASE,
) {
  // The key carries the MECHANISM, not the measured content: a digest of the
  // difference would change the moment one mechanism is partly fixed, and CI
  // would read the new key as a NEW failure instead of as progress.
  const byCell = new Map();
  for (const observation of observations) {
    for (const mechanism of observation.mechanisms ?? ["unclassified"])
      byCell.set(`${observation.method}|mech=${mechanism}`, true);
  }
  const entries = [];
  for (const cell of [...byCell.keys()].sort((left, right) =>
    left.localeCompare(right),
  )) {
    // The divergence COUNT is deliberately absent: two CI runs whose merge refs
    // share a `main` parent and differ by ten commits that touch no Rust moved
    // one file's hover count 91 -> 90 and 88 -> 90, which is 2 NEW + 2 STALE and
    // a red shard. It was sensitivity without direction — a shrink and a growth
    // are both one NEW and one STALE.
    const stage = phase === OPEN_PHASE ? "" : `|phase=${phase}`;
    entries.push(`aggregate:${fileId}|${cell}${stage}`);
  }
  return entries;
}

export function baselineRewriteReasons(
  selectedSuites,
  allSuites,
  selectedRepos,
  allRepos,
  narrowed = [],
) {
  return [
    selectedSuites.length !== allSuites.length ||
    allSuites.some((suite) => !selectedSuites.includes(suite))
      ? `--suites measured [${selectedSuites.join(", ")}], not all [${allSuites.join(", ")}] (FALSE-SHRINK)`
      : false,
    selectedRepos.length !== allRepos.length ||
    allRepos.some((repo) => !selectedRepos.includes(repo))
      ? `--corpus-repos measured [${selectedRepos.join(", ")}], not all [${allRepos.join(", ")}] (FALSE-SHRINK)`
      : false,
    ...narrowed,
  ];
}

export function assertNonemptySuites(cases, selectedSuites) {
  for (const suite of selectedSuites) {
    if (!cases.some((entry) => entry.suite === suite))
      throw new Error(`${suite} selected but discovered zero cases`);
  }
}

export function shardCorpusCases(cases, shard) {
  if (!shard) return cases;
  return cases.filter((entry) => {
    if (entry.suite !== "corpus") return true;
    return corpusShardIndex(entry.id, shard.count) === shard.index;
  });
}

export function corpusShardIndex(id, count) {
  return createHash("sha256").update(id).digest().readUInt32BE(0) % count;
}

export function selectKnownForScope(
  known,
  selectedSuites,
  selectedRepos,
  shard,
) {
  const scopes = selectedSuites.map((suite) => `${suite}/`);
  return known.filter((entry) => {
    const rest = entry.slice(entry.indexOf(":") + 1);
    if (!scopes.some((scope) => rest.startsWith(scope))) return false;
    if (!rest.startsWith("corpus/")) return true;
    if (!selectedRepos.some((repo) => rest.startsWith(`corpus/${repo}/`)))
      return false;
    if (!shard) return true;
    return (
      corpusShardIndex(rest.slice(0, rest.indexOf("|")), shard.count) ===
      shard.index
    );
  });
}
