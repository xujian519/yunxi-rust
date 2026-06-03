import { reviewData as mockReviewData } from '@/data/mockData';

export type ObjectionType = 'novelty' | 'inventive' | 'support' | string;

export interface ReviewObjection {
  id: string;
  type: ObjectionType;
  claim: string;
  citation: string;
  content: string;
}

export interface ReviewResponse {
  id: string;
  objectionId: string;
  content: string;
}

export interface ReviewData {
  objections: ReviewObjection[];
  responses: ReviewResponse[];
}

export function emptyReviewData(): ReviewData {
  return { objections: [], responses: [] };
}

export function serializeReviewData(data: ReviewData): string {
  return JSON.stringify(data, null, 2);
}

/** 从案件 review 文档或纯文本解析审查意见结构 */
export function parseReviewDocument(raw: string): ReviewData {
  const trimmed = raw.trim();
  if (!trimmed) return emptyReviewData();

  if (trimmed.startsWith('{')) {
    try {
      const parsed = JSON.parse(trimmed) as ReviewData;
      if (Array.isArray(parsed.objections)) {
        return {
          objections: parsed.objections,
          responses: Array.isArray(parsed.responses) ? parsed.responses : [],
        };
      }
    } catch {
      // fall through
    }
  }

  return parseReviewPlainText(trimmed);
}

function parseReviewPlainText(text: string): ReviewData {
  const blocks = text
    .split(/\n(?=\d+[.、)]\s*|【|审查意见|权利要求)/)
    .map((b) => b.trim())
    .filter(Boolean);

  if (blocks.length <= 1 && text.length > 40) {
    return {
      objections: [
        {
          id: 'obj-1',
          type: inferType(text),
          claim: '—',
          citation: '',
          content: text,
        },
      ],
      responses: [],
    };
  }

  const objections: ReviewObjection[] = blocks.map((block, i) => ({
    id: `obj-${i + 1}`,
    type: inferType(block),
    claim: extractClaim(block),
    citation: extractCitation(block),
    content: block,
  }));

  return { objections, responses: [] };
}

function inferType(text: string): ObjectionType {
  if (/新颖性|不具备新颖/.test(text)) return 'novelty';
  if (/创造性|显而易见|不具备创造/.test(text)) return 'inventive';
  if (/26条|支持|清楚|充分公开/.test(text)) return 'support';
  return 'novelty';
}

function extractClaim(text: string): string {
  const m = text.match(/权利要求\s*[\d、,-]+/);
  return m ? m[0] : '—';
}

function extractCitation(text: string): string {
  const m = text.match(/CN\d+[A-Z]?\d*|US\d+\/\d+|EP\d+[A-Z]?\d*/gi);
  return m ? m.join(' + ') : '';
}

export function defaultReviewData(): ReviewData {
  return {
    objections: [...mockReviewData.objections],
    responses: [...mockReviewData.responses],
  };
}

/** 将 OaParse 工具 JSON 输出转为 ReviewData */
export function oaParseJsonToReviewData(raw: string): ReviewData {
  try {
    const parsed = JSON.parse(raw) as {
      rejection_reasons?: Array<{
        type?: string;
        description?: string;
        affected_claims?: number[];
        cited_references?: string[];
      }>;
    };
    const reasons = parsed.rejection_reasons ?? [];
    if (reasons.length === 0) {
      return emptyReviewData();
    }
    const objections: ReviewObjection[] = reasons.map((r, i) => ({
      id: `obj-${i + 1}`,
      type: mapRejectionType(r.type ?? ''),
      claim:
        r.affected_claims && r.affected_claims.length > 0
          ? `权利要求${r.affected_claims.join('、')}`
          : '—',
      citation: (r.cited_references ?? []).join(' + '),
      content: r.description ?? '',
    }));
    return { objections, responses: [] };
  } catch {
    return emptyReviewData();
  }
}

function mapRejectionType(rejectionType: string): ObjectionType {
  const t = rejectionType.toLowerCase();
  if (t.includes('novelty') || t.includes('新颖')) return 'novelty';
  if (t.includes('inventive') || t.includes('creativ') || t.includes('创造')) return 'inventive';
  if (t.includes('support') || t.includes('clarity') || t.includes('支持') || t.includes('清楚')) {
    return 'support';
  }
  return 'novelty';
}
