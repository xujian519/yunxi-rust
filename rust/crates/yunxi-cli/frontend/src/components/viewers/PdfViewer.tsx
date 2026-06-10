import { useCallback, useEffect, useRef, useState } from 'react';
import * as pdfjsLib from 'pdfjs-dist';
import type { PDFDocumentProxy, PDFPageProxy, RenderTask } from 'pdfjs-dist';
import { useTheme } from '@/context/ThemeProvider';

pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  'pdfjs-dist/build/pdf.worker.min.mjs',
  import.meta.url,
).toString();

export interface PdfViewerProps {
  filePath: string;
  fileData?: Uint8Array;
}

interface Point {
  x: number;
  y: number;
}

interface HighlightAnnotation {
  type: 'highlight';
  page: number;
  rect: { x: number; y: number; width: number; height: number };
}

interface PenAnnotation {
  type: 'pen';
  page: number;
  points: Point[];
}

type Annotation = HighlightAnnotation | PenAnnotation;
type AnnotationMode = 'none' | 'highlight' | 'pen';

type TauriGlobal = {
  core?: {
    convertFileSrc?: (path: string) => string;
    invoke?: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
  };
};

function getTauriGlobal(): TauriGlobal | undefined {
  return (window as unknown as { __TAURI__?: TauriGlobal }).__TAURI__;
}

async function resolvePdfData(filePath: string, fileData?: Uint8Array): Promise<Uint8Array> {
  if (fileData) return fileData;

  if (filePath.startsWith('http://') || filePath.startsWith('https://') || filePath.startsWith('blob:')) {
    const res = await fetch(filePath);
    if (!res.ok) throw new Error(`HTTP ${res.status}: ${res.statusText}`);
    return new Uint8Array(await res.arrayBuffer());
  }

  const tauri = getTauriGlobal();
  if (tauri?.core?.convertFileSrc) {
    const url = tauri.core.convertFileSrc(filePath);
    const res = await fetch(url);
    if (!res.ok) throw new Error(`HTTP ${res.status}: ${res.statusText}`);
    return new Uint8Array(await res.arrayBuffer());
  }

  try {
    const res = await fetch(filePath);
    if (res.ok) return new Uint8Array(await res.arrayBuffer());
  } catch {
    // fall through
  }

  throw new Error(`无法读取 PDF 文件: ${filePath}`);
}

export default function PdfViewer({ filePath, fileData }: PdfViewerProps) {
  const { resolved: theme } = useTheme();
  const isDark = theme === 'dark';

  const [pdfDoc, setPdfDoc] = useState<PDFDocumentProxy | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [scale, setScale] = useState(1.2);
  const [fitWidth, setFitWidth] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [annotations, setAnnotations] = useState<Annotation[]>([]);
  const [annotationMode, setAnnotationMode] = useState<AnnotationMode>('none');

  const containerRef = useRef<HTMLDivElement>(null);
  const pdfCanvasRef = useRef<HTMLCanvasElement>(null);
  const annoCanvasRef = useRef<HTMLCanvasElement>(null);
  const pageRef = useRef<PDFPageProxy | null>(null);
  const renderTaskRef = useRef<RenderTask | null>(null);

  const drawingRef = useRef<{
    active: boolean;
    startX: number;
    startY: number;
    points: Point[];
  }>({ active: false, startX: 0, startY: 0, points: [] });

  // Load PDF document
  useEffect(() => {
    let cancelled = false;

    async function load() {
      setIsLoading(true);
      setError(null);
      try {
        const data = await resolvePdfData(filePath, fileData);
        if (cancelled) return;
        const pdf = await pdfjsLib.getDocument({ data }).promise;
        if (cancelled) {
          return;
        }
        setPdfDoc(pdf);
        setCurrentPage(1);
      } catch (err) {
        if (!cancelled) {
          const msg = err instanceof Error ? err.message : String(err);
          setError(msg);
        }
      } finally {
        if (!cancelled) setIsLoading(false);
      }
    }

    load();
    return () => {
      cancelled = true;
    };
  }, [filePath, fileData]);

  // Cleanup PDF on unmount
  useEffect(() => {
    return () => {
      // pdfjs-dist v4+ cleanup if needed
    };
  }, [pdfDoc]);

  const getPageScale = useCallback(
    async (page: PDFPageProxy, desiredScale: number, fit: boolean): Promise<number> => {
      if (!fit) return desiredScale;
      const viewport = page.getViewport({ scale: 1 });
      const containerWidth = containerRef.current?.clientWidth ?? viewport.width;
      const padding = 32;
      return (containerWidth - padding) / viewport.width;
    },
    [],
  );

  // Render current page
  useEffect(() => {
    if (!pdfDoc) return;

    let cancelled = false;

    async function render() {
      const pdfCanvas = pdfCanvasRef.current;
      const annoCanvas = annoCanvasRef.current;
      if (!pdfCanvas || !annoCanvas) return;

      setIsLoading(true);
      try {
        renderTaskRef.current?.cancel();
        if (!pdfDoc) return;
        const page = await pdfDoc.getPage(currentPage);
        if (cancelled) {
          return;
        }
        pageRef.current = page;
        const actualScale = await getPageScale(page, scale, fitWidth);
        if (cancelled) {
          return;
        }
        const viewport = page.getViewport({ scale: actualScale });
        pdfCanvas.width = viewport.width;
        pdfCanvas.height = viewport.height;
        annoCanvas.width = viewport.width;
        annoCanvas.height = viewport.height;
        const ctx = pdfCanvas.getContext('2d');
        if (!ctx) return;
        // @ts-ignore pdfjs-dist v4 API
        const task = page.render({ canvasContext: ctx, viewport });

        await task.promise;
        if (cancelled) return;

        // Render annotations for current page
        renderAnnotations(page, actualScale);
      } catch (err) {
        if (!cancelled) {
          const msg = err instanceof Error ? err.message : String(err);
          if (!msg.includes('RenderingCancelled')) {
            setError(msg);
          }
        }
      } finally {
        if (!cancelled) setIsLoading(false);
      }
    }

    render();

    return () => {
      cancelled = true;
      renderTaskRef.current?.cancel();
    };
  }, [pdfDoc, currentPage, scale, fitWidth, getPageScale, annotations]);

  const renderAnnotations = useCallback(
    (_page: PDFPageProxy, actualScale: number) => {
      const annoCanvas = annoCanvasRef.current;
      if (!annoCanvas) return;

      const ctx = annoCanvas.getContext('2d');
      if (!ctx) return;

      ctx.clearRect(0, 0, annoCanvas.width, annoCanvas.height);

      const pageAnno = annotations.filter((a) => a.page === currentPage);

      for (const anno of pageAnno) {
        if (anno.type === 'highlight') {
          ctx.fillStyle = '#FFEB3B80';
          ctx.fillRect(
            anno.rect.x * actualScale,
            anno.rect.y * actualScale,
            anno.rect.width * actualScale,
            anno.rect.height * actualScale,
          );
        } else if (anno.type === 'pen') {
          ctx.strokeStyle = '#FF0000';
          ctx.lineWidth = 2;
          ctx.lineCap = 'round';
          ctx.lineJoin = 'round';
          ctx.beginPath();
          for (let i = 0; i < anno.points.length; i++) {
            const p = anno.points[i];
            const x = p.x * actualScale;
            const y = p.y * actualScale;
            if (i === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
          }
          ctx.stroke();
        }
      }
    },
    [annotations, currentPage],
  );

  // Handle resize for fit-width mode
  useEffect(() => {
    if (!fitWidth || !pdfDoc) return;

    const observer = new ResizeObserver(() => {
      // Trigger re-render by toggling a dummy state or reusing existing deps
      // Since scale is not used in fitWidth mode, we can just force a re-render
      // by updating a timestamp or by calling render directly.
      // Simpler: we just call the render effect by updating a counter.
      setScale((s) => s);
    });

    if (containerRef.current) {
      observer.observe(containerRef.current);
    }

    return () => observer.disconnect();
  }, [fitWidth, pdfDoc]);

  const getMousePos = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>): Point => {
      const canvas = annoCanvasRef.current;
      if (!canvas) return { x: 0, y: 0 };
      const rect = canvas.getBoundingClientRect();
      return {
        x: e.clientX - rect.left,
        y: e.clientY - rect.top,
      };
    },
    [],
  );

  const getPdfCoordinate = useCallback(
    (point: Point, actualScale: number): Point => ({
      x: point.x / actualScale,
      y: point.y / actualScale,
    }),
    [],
  );

  const getCurrentScale = useCallback(async (): Promise<number> => {
    if (!pageRef.current) return scale;
    return getPageScale(pageRef.current, scale, fitWidth);
  }, [scale, fitWidth, getPageScale]);

  const handleMouseDown = useCallback(
    async (e: React.MouseEvent<HTMLCanvasElement>) => {
      if (annotationMode === 'none') return;
      e.preventDefault();

      const pos = getMousePos(e);
      const actualScale = await getCurrentScale();

      drawingRef.current = {
        active: true,
        startX: pos.x,
        startY: pos.y,
        points: [getPdfCoordinate(pos, actualScale)],
      };
    },
    [annotationMode, getMousePos, getCurrentScale, getPdfCoordinate],
  );

  const handleMouseMove = useCallback(
    async (e: React.MouseEvent<HTMLCanvasElement>) => {
      if (!drawingRef.current.active || annotationMode === 'none') return;
      e.preventDefault();

      const pos = getMousePos(e);
      const actualScale = await getCurrentScale();
      const pdfPos = getPdfCoordinate(pos, actualScale);

      if (annotationMode === 'pen') {
        drawingRef.current.points.push(pdfPos);

        // Live preview on annotation canvas
        const canvas = annoCanvasRef.current;
        const ctx = canvas?.getContext('2d');
        if (ctx && canvas) {
          ctx.strokeStyle = '#FF0000';
          ctx.lineWidth = 2;
          ctx.lineCap = 'round';
          ctx.lineJoin = 'round';
          const pts = drawingRef.current.points;
          if (pts.length >= 2) {
            const prev = pts[pts.length - 2];
            const curr = pts[pts.length - 1];
            ctx.beginPath();
            ctx.moveTo(prev.x * actualScale, prev.y * actualScale);
            ctx.lineTo(curr.x * actualScale, curr.y * actualScale);
            ctx.stroke();
          }
        }
      } else if (annotationMode === 'highlight') {
        // Live preview highlight rect
        const canvas = annoCanvasRef.current;
        const ctx = canvas?.getContext('2d');
        if (ctx && canvas) {
          renderAnnotations(pageRef.current!, actualScale);
          ctx.fillStyle = '#FFEB3B80';
          const startX = Math.min(drawingRef.current.startX, pos.x);
          const startY = Math.min(drawingRef.current.startY, pos.y);
          const width = Math.abs(pos.x - drawingRef.current.startX);
          const height = Math.abs(pos.y - drawingRef.current.startY);
          ctx.fillRect(startX, startY, width, height);
        }
      }
    },
    [annotationMode, getMousePos, getCurrentScale, getPdfCoordinate, renderAnnotations],
  );

  const handleMouseUp = useCallback(
    async (e: React.MouseEvent<HTMLCanvasElement>) => {
      if (!drawingRef.current.active || annotationMode === 'none') return;
      e.preventDefault();

      const pos = getMousePos(e);
      const actualScale = await getCurrentScale();
      const pdfPos = getPdfCoordinate(pos, actualScale);

      if (annotationMode === 'highlight') {
        const startPdf = getPdfCoordinate(
          { x: drawingRef.current.startX, y: drawingRef.current.startY },
          actualScale,
        );
        const x = Math.min(startPdf.x, pdfPos.x);
        const y = Math.min(startPdf.y, pdfPos.y);
        const width = Math.abs(pdfPos.x - startPdf.x);
        const height = Math.abs(pdfPos.y - startPdf.y);

        if (width > 2 && height > 2) {
          setAnnotations((prev) => [
            ...prev,
            { type: 'highlight', page: currentPage, rect: { x, y, width, height } },
          ]);
        }
      } else if (annotationMode === 'pen') {
        if (drawingRef.current.points.length >= 2) {
          setAnnotations((prev) => [
            ...prev,
            { type: 'pen', page: currentPage, points: [...drawingRef.current.points] },
          ]);
        }
      }

      drawingRef.current.active = false;

      // Re-render to show final annotation state
      if (pageRef.current) {
        renderAnnotations(pageRef.current, actualScale);
      }
    },
    [
      annotationMode,
      getMousePos,
      getCurrentScale,
      getPdfCoordinate,
      currentPage,
      renderAnnotations,
    ],
  );

  const handlePrevPage = useCallback(() => {
    setCurrentPage((p) => Math.max(1, p - 1));
  }, []);

  const handleNextPage = useCallback(() => {
    setCurrentPage((p) => Math.min(pdfDoc?.numPages ?? 1, p + 1));
  }, [pdfDoc?.numPages]);

  const handlePageInput = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const val = parseInt(e.target.value, 10);
      if (!isNaN(val) && pdfDoc) {
        setCurrentPage(Math.max(1, Math.min(pdfDoc.numPages, val)));
      }
    },
    [pdfDoc],
  );

  const handleZoomIn = useCallback(() => {
    setFitWidth(false);
    setScale((s) => Math.min(3, s + 0.2));
  }, []);

  const handleZoomOut = useCallback(() => {
    setFitWidth(false);
    setScale((s) => Math.max(0.3, s - 0.2));
  }, []);

  const handleFitWidth = useCallback(() => {
    setFitWidth(true);
  }, []);

  const toolbarBg = isDark ? '#1e1e1e' : '#f5f5f5';
  const toolbarBorder = isDark ? '#333' : '#ddd';
  const textColor = isDark ? '#e0e0e0' : '#333';
  const btnHoverBg = isDark ? '#333' : '#e0e0e0';
  const inputBg = isDark ? '#2a2a2a' : '#fff';

  const activeBtnStyle = (mode: AnnotationMode): React.CSSProperties => ({
    padding: '6px 12px',
    border: '1px solid',
    borderColor: annotationMode === mode ? '#2196F3' : toolbarBorder,
    borderRadius: 4,
    background: annotationMode === mode ? (isDark ? '#1a3a5c' : '#e3f2fd') : 'transparent',
    color: annotationMode === mode ? '#2196F3' : textColor,
    cursor: 'pointer',
    fontSize: 13,
  });

  if (error) {
    return (
      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          height: '100%',
          padding: 24,
          color: isDark ? '#ff6b6b' : '#c0392b',
          background: isDark ? '#1a1a1a' : '#fff',
          fontFamily: 'system-ui, sans-serif',
        }}
      >
        <h3 style={{ marginBottom: 8 }}>PDF 加载失败</h3>
        <p style={{ fontSize: 14, opacity: 0.8 }}>{error}</p>
      </div>
    );
  }

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        width: '100%',
        background: isDark ? '#121212' : '#f0f0f0',
        fontFamily: 'system-ui, sans-serif',
      }}
    >
      {/* Toolbar */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 8,
          padding: '8px 12px',
          background: toolbarBg,
          borderBottom: `1px solid ${toolbarBorder}`,
          flexWrap: 'wrap',
        }}
      >
        {/* Page navigation */}
        <button
          onClick={handlePrevPage}
          disabled={currentPage <= 1}
          style={{
            padding: '6px 10px',
            border: `1px solid ${toolbarBorder}`,
            borderRadius: 4,
            background: 'transparent',
            color: textColor,
            cursor: currentPage <= 1 ? 'not-allowed' : 'pointer',
            opacity: currentPage <= 1 ? 0.5 : 1,
            fontSize: 13,
          }}
          onMouseEnter={(e) => {
            if (currentPage > 1) (e.target as HTMLButtonElement).style.background = btnHoverBg;
          }}
          onMouseLeave={(e) => {
            (e.target as HTMLButtonElement).style.background = 'transparent';
          }}
        >
          ◀
        </button>
        <span style={{ color: textColor, fontSize: 13 }}>
          <input
            type="number"
            value={currentPage}
            min={1}
            max={pdfDoc?.numPages ?? 1}
            onChange={handlePageInput}
            style={{
              width: 48,
              padding: '4px 6px',
              border: `1px solid ${toolbarBorder}`,
              borderRadius: 4,
              background: inputBg,
              color: textColor,
              fontSize: 13,
              textAlign: 'center',
            }}
          />
          {' / '}
          {pdfDoc?.numPages ?? '-'}
        </span>
        <button
          onClick={handleNextPage}
          disabled={!pdfDoc || currentPage >= pdfDoc.numPages}
          style={{
            padding: '6px 10px',
            border: `1px solid ${toolbarBorder}`,
            borderRadius: 4,
            background: 'transparent',
            color: textColor,
            cursor: !pdfDoc || currentPage >= pdfDoc.numPages ? 'not-allowed' : 'pointer',
            opacity: !pdfDoc || currentPage >= pdfDoc.numPages ? 0.5 : 1,
            fontSize: 13,
          }}
          onMouseEnter={(e) => {
            if (pdfDoc && currentPage < pdfDoc.numPages)
              (e.target as HTMLButtonElement).style.background = btnHoverBg;
          }}
          onMouseLeave={(e) => {
            (e.target as HTMLButtonElement).style.background = 'transparent';
          }}
        >
          ▶
        </button>

        <div style={{ width: 1, height: 20, background: toolbarBorder, margin: '0 4px' }} />

        {/* Zoom controls */}
        <button
          onClick={handleZoomOut}
          style={{
            padding: '6px 10px',
            border: `1px solid ${toolbarBorder}`,
            borderRadius: 4,
            background: 'transparent',
            color: textColor,
            cursor: 'pointer',
            fontSize: 13,
          }}
          onMouseEnter={(e) => {
            (e.target as HTMLButtonElement).style.background = btnHoverBg;
          }}
          onMouseLeave={(e) => {
            (e.target as HTMLButtonElement).style.background = 'transparent';
          }}
        >
          －
        </button>
        <span style={{ color: textColor, fontSize: 13, minWidth: 48, textAlign: 'center' }}>
          {fitWidth ? '适应宽度' : `${Math.round(scale * 100)}%`}
        </span>
        <button
          onClick={handleZoomIn}
          style={{
            padding: '6px 10px',
            border: `1px solid ${toolbarBorder}`,
            borderRadius: 4,
            background: 'transparent',
            color: textColor,
            cursor: 'pointer',
            fontSize: 13,
          }}
          onMouseEnter={(e) => {
            (e.target as HTMLButtonElement).style.background = btnHoverBg;
          }}
          onMouseLeave={(e) => {
            (e.target as HTMLButtonElement).style.background = 'transparent';
          }}
        >
          ＋
        </button>
        <button
          onClick={handleFitWidth}
          style={{
            padding: '6px 10px',
            border: `1px solid ${toolbarBorder}`,
            borderRadius: 4,
            background: fitWidth ? (isDark ? '#1a3a5c' : '#e3f2fd') : 'transparent',
            color: fitWidth ? '#2196F3' : textColor,
            cursor: 'pointer',
            fontSize: 13,
          }}
          onMouseEnter={(e) => {
            if (!fitWidth) (e.target as HTMLButtonElement).style.background = btnHoverBg;
          }}
          onMouseLeave={(e) => {
            if (!fitWidth) (e.target as HTMLButtonElement).style.background = 'transparent';
          }}
        >
          适应宽度
        </button>

        <div style={{ width: 1, height: 20, background: toolbarBorder, margin: '0 4px' }} />

        {/* Annotation mode */}
        <button onClick={() => setAnnotationMode('none')} style={activeBtnStyle('none')}>
          无标注
        </button>
        <button onClick={() => setAnnotationMode('highlight')} style={activeBtnStyle('highlight')}>
          高亮
        </button>
        <button onClick={() => setAnnotationMode('pen')} style={activeBtnStyle('pen')}>
          画笔
        </button>

        {annotations.length > 0 && (
          <button
            onClick={() => setAnnotations([])}
            style={{
              padding: '6px 12px',
              border: `1px solid ${toolbarBorder}`,
              borderRadius: 4,
              background: 'transparent',
              color: '#ff6b6b',
              cursor: 'pointer',
              fontSize: 13,
              marginLeft: 'auto',
            }}
          >
            清除标注
          </button>
        )}
      </div>

      {/* Canvas container */}
      <div
        ref={containerRef}
        style={{
          flex: 1,
          overflow: 'auto',
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'flex-start',
          padding: 16,
          position: 'relative',
        }}
      >
        {isLoading && (
          <div
            style={{
              position: 'absolute',
              top: '50%',
              left: '50%',
              transform: 'translate(-50%, -50%)',
              color: textColor,
              fontSize: 14,
              zIndex: 2,
            }}
          >
            加载中…
          </div>
        )}

        <div style={{ position: 'relative' }}>
          <canvas
            ref={pdfCanvasRef}
            style={{
              display: 'block',
              boxShadow: '0 2px 8px rgba(0,0,0,0.15)',
            }}
          />
          <canvas
            ref={annoCanvasRef}
            onMouseDown={handleMouseDown}
            onMouseMove={handleMouseMove}
            onMouseUp={handleMouseUp}
            onMouseLeave={handleMouseUp}
            style={{
              position: 'absolute',
              top: 0,
              left: 0,
              cursor: annotationMode === 'none' ? 'default' : 'crosshair',
              display: 'block',
            }}
          />
        </div>
      </div>
    </div>
  );
}
