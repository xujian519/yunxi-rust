export type DiffLineType = 'add' | 'del' | 'unchanged';

export interface DiffLine {
  type: DiffLineType;
  lineNum: number;
  content: string;
}

type DiffOp =
  | { kind: 'equal'; line: string }
  | { kind: 'del'; line: string }
  | { kind: 'add'; line: string };

/** 基于 LCS 的行级 diff，输出左右分栏（与 CompareView 一致） */
export function buildSideBySideDiff(
  originalText: string,
  modifiedText: string,
): { original: DiffLine[]; modified: DiffLine[] } {
  const a = originalText.replace(/\r\n/g, '\n').split('\n');
  const b = modifiedText.replace(/\r\n/g, '\n').split('\n');
  const ops = diffOps(a, b);

  const original: DiffLine[] = [];
  const modified: DiffLine[] = [];
  let leftNum = 0;
  let rightNum = 0;

  for (const op of ops) {
    if (op.kind === 'equal') {
      leftNum += 1;
      rightNum += 1;
      original.push({ type: 'unchanged', lineNum: leftNum, content: op.line });
      modified.push({ type: 'unchanged', lineNum: rightNum, content: op.line });
    } else if (op.kind === 'del') {
      leftNum += 1;
      original.push({ type: 'del', lineNum: leftNum, content: op.line });
    } else {
      rightNum += 1;
      modified.push({ type: 'add', lineNum: rightNum, content: op.line });
    }
  }

  return { original, modified };
}

function diffOps(a: string[], b: string[]): DiffOp[] {
  const m = a.length;
  const n = b.length;
  const dp: number[][] = Array.from({ length: m + 1 }, () => new Array<number>(n + 1).fill(0));

  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      if (a[i - 1] === b[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1;
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
      }
    }
  }

  const ops: DiffOp[] = [];
  let i = m;
  let j = n;
  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && a[i - 1] === b[j - 1]) {
      ops.push({ kind: 'equal', line: a[i - 1] });
      i -= 1;
      j -= 1;
    } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
      ops.push({ kind: 'add', line: b[j - 1] });
      j -= 1;
    } else {
      ops.push({ kind: 'del', line: a[i - 1] });
      i -= 1;
    }
  }

  ops.reverse();
  return ops;
}
