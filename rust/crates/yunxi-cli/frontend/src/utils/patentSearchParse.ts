import type { SearchResult } from '@/data/mockData';

export interface PatentSearchRow {
  patent_name?: string;
  application_number?: string;
  applicant?: string;
  ipc_main_class?: string;
  application_date?: string;
  abstract?: string;
}

export interface PatentSearchResponse {
  status: string;
  message?: string;
  query?: string;
  total?: number;
  results?: PatentSearchRow[];
}

export interface ParsedPatentSearch {
  items: SearchResult[];
  unavailable?: string;
  error?: string;
}

function inferStatus(ipc?: string): SearchResult['status'] {
  const c = (ipc || '').trim().charAt(0).toUpperCase();
  if (c === 'A') return 'examination';
  if (c === 'U') return 'published';
  if (c === 'S') return 'published';
  return 'published';
}

function rowToResult(row: PatentSearchRow, index: number): SearchResult {
  const total = 10;
  const relevance = Math.max(0.35, 1 - index * (0.55 / total));
  return {
    id: row.application_number || `patent-${index}`,
    title: row.patent_name?.trim() || '（无名称）',
    number: row.application_number?.trim() || '—',
    applicant: row.applicant?.trim() || '—',
    date: row.application_date?.trim() || '—',
    status: inferStatus(row.ipc_main_class),
    abstract: row.abstract?.trim() || '（无摘要）',
    relevance,
  };
}

/** 解析 PatentSearch 工具返回的 JSON 字符串 */
export function parsePatentSearchResults(raw: string): ParsedPatentSearch {
  const trimmed = raw.trim();
  if (!trimmed) {
    return { items: [], error: '检索结果为空' };
  }

  if (!trimmed.startsWith('{') && !trimmed.startsWith('[')) {
    return { items: [], error: trimmed.slice(0, 400) };
  }

  try {
    const data = JSON.parse(trimmed) as PatentSearchResponse;
    if (data.status === 'unavailable') {
      return {
        items: [],
        unavailable: data.message || 'patent_db 不可用，请检查 PostgreSQL 与 ~/.infra/infra.env',
      };
    }
    if (data.status !== 'ok') {
      return { items: [], error: data.message || '检索返回异常状态' };
    }
    const items = (data.results ?? []).map(rowToResult);
    return { items };
  } catch {
    return { items: [], error: '无法解析检索结果 JSON' };
  }
}

/** 按 UI 筛选标签过滤（IPC 大类粗分） */
export function filterByPatentType(
  items: SearchResult[],
  filter: string,
  rows?: PatentSearchRow[],
): SearchResult[] {
  if (filter === '全部' || filter === 'PCT') return items;
  if (!rows || rows.length === 0) return items;

  const ipcOf = (i: number) => (rows[i]?.ipc_main_class || '').trim().toUpperCase();

  return items.filter((_, i) => {
    const ipc = ipcOf(i);
    switch (filter) {
      case '发明专利':
        return ipc.startsWith('A');
      case '实用新型':
        return ipc.startsWith('U');
      case '外观设计':
        return ipc.startsWith('S') || ipc.startsWith('D');
      default:
        return true;
    }
  });
}
