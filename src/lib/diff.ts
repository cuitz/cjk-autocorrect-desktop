export type DiffType = "equal" | "add" | "change";

export interface DiffSegment {
  type: DiffType;
  text: string;
}

/**
 * Compute character-level diff between the original and formatted text.
 * Uses Hirschberg's divide-and-conquer algorithm — O(n·m) time, O(min(n,m)) space.
 * Adjacent remove+add pairs are merged into "change" segments.
 */
export function diffChars(original: string, formatted: string): DiffSegment[] {
  const oldChars = [...original];
  const newChars = [...formatted];

  const raw = hirschberg(oldChars, newChars);
  return mergeOps(raw, newChars);
}

// ── Hirschberg: linear-space LCS → edit script ──

type RawOp = 0 | 1 | -1; // 0=equal, 1=add, -1=remove

function hirschberg(a: string[], b: string[]): RawOp[] {
  const n = a.length;
  const m = b.length;

  // Base cases: use full O(n×m) DP when strings are small enough
  if (n === 0) return Array<RawOp>(m).fill(1);
  if (m === 0) return Array<RawOp>(n).fill(-1);
  if (n * m < 1000) return lcsFull(a, b);

  // Divide: find a midpoint that lies on an optimal path
  const amid = n >> 1;
  const L1 = lcsRow(a.slice(0, amid), b);        // forward from top-left
  const L2 = lcsRowRev(a.slice(amid), b);         // reverse from bottom-right

  // bmid = k that maximizes L1[k] + L2[m - k]
  let best = -1;
  let bmid = 0;
  for (let k = 0; k <= m; k++) {
    const val = L1[k] + L2[m - k];
    if (val > best) {
      best = val;
      bmid = k;
    }
  }

  // Conquer: recursively solve the two sub-problems
  const left = hirschberg(a.slice(0, amid), b.slice(0, bmid));
  const right = hirschberg(a.slice(amid), b.slice(bmid));
  return [...left, ...right];
}

// Compute LCS lengths for a prefix vs all of b (returns last row only)
function lcsRow(a: string[], b: string[]): number[] {
  const m = b.length;
  let prev = new Array<number>(m + 1).fill(0);
  let curr = new Array<number>(m + 1).fill(0);

  for (let i = 1; i <= a.length; i++) {
    curr[0] = 0;
    for (let j = 1; j <= m; j++) {
      if (a[i - 1] === b[j - 1]) {
        curr[j] = prev[j - 1] + 1;
      } else {
        curr[j] = Math.max(prev[j], curr[j - 1]);
      }
    }
    const tmp = prev;
    prev = curr;
    curr = tmp;
  }

  return prev;
}

// Compute LCS lengths for a suffix vs all of b, in reverse direction
function lcsRowRev(a: string[], b: string[]): number[] {
  const m = b.length;
  let prev = new Array<number>(m + 1).fill(0);
  let curr = new Array<number>(m + 1).fill(0);

  for (let i = a.length - 1; i >= 0; i--) {
    curr[m] = 0;
    for (let j = m - 1; j >= 0; j--) {
      if (a[i] === b[j]) {
        curr[j] = prev[j + 1] + 1;
      } else {
        curr[j] = Math.max(prev[j], curr[j + 1]);
      }
    }
    const tmp = prev;
    prev = curr;
    curr = tmp;
  }

  return prev;
}

// Full O(n×m) DP for small substrings (n*m < 1000)
function lcsFull(a: string[], b: string[]): RawOp[] {
  const n = a.length;
  const m = b.length;
  const table: number[][] = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));

  for (let i = 1; i <= n; i++) {
    for (let j = 1; j <= m; j++) {
      if (a[i - 1] === b[j - 1]) {
        table[i][j] = table[i - 1][j - 1] + 1;
      } else {
        table[i][j] = Math.max(table[i - 1][j], table[i][j - 1]);
      }
    }
  }

  const raw: RawOp[] = [];
  let i = n, j = m;
  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && a[i - 1] === b[j - 1]) {
      raw.push(0);
      i--;
      j--;
    } else if (j > 0 && (i === 0 || table[i][j - 1] >= table[i - 1][j])) {
      raw.push(1);
      j--;
    } else {
      raw.push(-1);
      i--;
    }
  }

  return raw.reverse();
}

// ── Merge raw ops into DiffSegments ──

function mergeOps(raw: RawOp[], newChars: string[]): DiffSegment[] {
  const segments: DiffSegment[] = [];
  let i = 0;
  let newIdx = 0;

  while (i < raw.length) {
    if (raw[i] === 0) {
      let j = i;
      while (j < raw.length && raw[j] === 0) j++;
      segments.push({ type: "equal", text: newChars.slice(newIdx, newIdx + (j - i)).join("") });
      newIdx += j - i;
      i = j;
    } else if (raw[i] === 1) {
      const buf: string[] = [];
      while (i < raw.length && raw[i] === 1) {
        buf.push(newChars[newIdx]);
        newIdx++;
        i++;
      }
      segments.push({ type: "add", text: buf.join("") });
    } else {
      // Remove — may precede an add → merge into "change"
      let rCount = 0;
      while (i < raw.length && raw[i] === -1) {
        rCount++;
        i++;
      }
      let aCount = 0;
      const addStart = newIdx;
      while (i < raw.length && raw[i] === 1) {
        aCount++;
        newIdx++;
        i++;
      }
      if (aCount > 0) {
        segments.push({ type: "change", text: newChars.slice(addStart, addStart + aCount).join("") });
      }
    }
  }

  return segments;
}
