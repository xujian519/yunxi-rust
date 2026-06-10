import React, { useEffect, useState, useCallback, useRef } from "react";
import * as XLSX from "xlsx";

interface ExcelViewerProps {
  filePath: string;
  fileData?: ArrayBuffer;
}

interface CellEditState {
  row: number;
  col: number;
  value: string;
}

export default function ExcelViewer({ filePath, fileData }: ExcelViewerProps) {
  const [workbook, setWorkbook] = useState<XLSX.WorkBook | null>(null);
  const [sheetNames, setSheetNames] = useState<string[]>([]);
  const [currentSheet, setCurrentSheet] = useState<string>("");
  const [sheetData, setSheetData] = useState<unknown[][]>([]);
  const [error, setError] = useState<string>("");
  const [loading, setLoading] = useState<boolean>(false);
  const [editingCell, setEditingCell] = useState<CellEditState | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const readFile = useCallback(async (): Promise<ArrayBuffer> => {
    if (fileData) {
      return fileData;
    }
    const response = await fetch(filePath);
    if (!response.ok) {
      throw new Error(`Failed to fetch file: ${response.status} ${response.statusText}`);
    }
    return response.arrayBuffer();
  }, [filePath, fileData]);

  const loadWorkbook = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      const data = await readFile();
      if (data.byteLength === 0) {
        throw new Error("File is empty");
      }
      const wb = XLSX.read(data, { type: "array" });
      if (!wb.SheetNames || wb.SheetNames.length === 0) {
        throw new Error("No sheets found in file");
      }
      setWorkbook(wb);
      setSheetNames(wb.SheetNames);
      setCurrentSheet(wb.SheetNames[0]);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(`Failed to load Excel file: ${msg}`);
      setWorkbook(null);
      setSheetNames([]);
      setCurrentSheet("");
      setSheetData([]);
    } finally {
      setLoading(false);
    }
  }, [readFile]);

  useEffect(() => {
    loadWorkbook();
  }, [loadWorkbook]);

  useEffect(() => {
    if (!workbook || !currentSheet) {
      setSheetData([]);
      return;
    }
    const sheet = workbook.Sheets[currentSheet];
    if (!sheet) {
      setSheetData([]);
      return;
    }
    const json = XLSX.utils.sheet_to_json(sheet, { header: 1, defval: "" });
    setSheetData(json as unknown[][]);
  }, [workbook, currentSheet]);

  useEffect(() => {
    if (editingCell && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [editingCell]);

  const getCellValue = useCallback(
    (row: number, col: number): string => {
      if (!workbook || !currentSheet) return "";
      const sheet = workbook.Sheets[currentSheet];
      if (!sheet) return "";
      const cellRef = XLSX.utils.encode_cell({ r: row, c: col });
      const cell = sheet[cellRef];
      if (!cell) return "";
      return cell.w !== undefined ? String(cell.w) : String(cell.v ?? "");
    },
    [workbook, currentSheet]
  );

  const setCellValue = useCallback(
    (row: number, col: number, value: string) => {
      if (!workbook || !currentSheet) return;
      const sheet = workbook.Sheets[currentSheet];
      if (!sheet) return;
      const cellRef = XLSX.utils.encode_cell({ r: row, c: col });
      const cell = sheet[cellRef];
      if (cell) {
        cell.v = value;
        cell.w = value;
      } else {
        sheet[cellRef] = { v: value, w: value, t: "s" };
      }
      // Update sheet range to include new cell if necessary
      const range = XLSX.utils.decode_range(sheet["!ref"] || "A1");
      let updated = false;
      if (row > range.e.r) {
        range.e.r = row;
        updated = true;
      }
      if (col > range.e.c) {
        range.e.c = col;
        updated = true;
      }
      if (updated) {
        sheet["!ref"] = XLSX.utils.encode_range(range);
      }
      // Refresh displayed data
      const json = XLSX.utils.sheet_to_json(sheet, { header: 1, defval: "" });
      setSheetData(json as unknown[][]);
      setWorkbook({ ...workbook });
    },
    [workbook, currentSheet]
  );

  const handleDoubleClick = useCallback(
    (row: number, col: number) => {
      const value = getCellValue(row, col);
      setEditingCell({ row, col, value });
    },
    [getCellValue]
  );

  const handleEditSave = useCallback(() => {
    if (!editingCell) return;
    setCellValue(editingCell.row, editingCell.col, editingCell.value);
    setEditingCell(null);
  }, [editingCell, setCellValue]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Enter") {
        handleEditSave();
      } else if (e.key === "Escape") {
        setEditingCell(null);
      }
    },
    [handleEditSave]
  );

  const handleExport = useCallback(() => {
    if (!workbook) return;
    const fileName = filePath.split("/").pop() || "export.xlsx";
    XLSX.writeFile(workbook, fileName);
  }, [workbook, filePath]);

  const getColumnWidth = useCallback(
    (colIndex: number): number => {
      if (sheetData.length === 0) return 100;
      let maxLen = 0;
      for (let i = 0; i < sheetData.length; i++) {
        const row = sheetData[i];
        if (row && row[colIndex] !== undefined) {
          const len = String(row[colIndex]).length;
          if (len > maxLen) maxLen = len;
        }
      }
      return Math.max(60, Math.min(300, maxLen * 8 + 16));
    },
    [sheetData]
  );

  if (loading) {
    return (
      <div style={{ padding: 20, fontFamily: "sans-serif" }}>
        Loading Excel file...
      </div>
    );
  }

  if (error) {
    return (
      <div
        style={{
          padding: 20,
          fontFamily: "sans-serif",
          color: "#d32f2f",
          backgroundColor: "#ffebee",
          borderRadius: 4,
        }}
      >
        {error}
      </div>
    );
  }

  if (!workbook || sheetData.length === 0) {
    return (
      <div style={{ padding: 20, fontFamily: "sans-serif" }}>
        No data to display
      </div>
    );
  }

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        fontFamily: "sans-serif",
        fontSize: 14,
      }}
    >
      {/* Toolbar */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "8px 12px",
          borderBottom: "1px solid #ddd",
          backgroundColor: "#fafafa",
        }}
      >
        <div style={{ display: "flex", gap: 4 }}>
          {sheetNames.map((name) => (
            <button
              key={name}
              onClick={() => setCurrentSheet(name)}
              style={{
                padding: "6px 14px",
                border: "1px solid",
                borderColor: currentSheet === name ? "#1976d2" : "#ddd",
                backgroundColor:
                  currentSheet === name ? "#e3f2fd" : "#fff",
                color: currentSheet === name ? "#1976d2" : "#333",
                borderRadius: 4,
                cursor: "pointer",
                fontSize: 13,
                fontWeight: currentSheet === name ? 600 : 400,
              }}
            >
              {name}
            </button>
          ))}
        </div>
        <button
          onClick={handleExport}
          style={{
            padding: "6px 14px",
            border: "1px solid #1976d2",
            backgroundColor: "#1976d2",
            color: "#fff",
            borderRadius: 4,
            cursor: "pointer",
            fontSize: 13,
          }}
        >
          Export
        </button>
      </div>

      {/* Table */}
      <div style={{ flex: 1, overflow: "auto" }}>
        <table
          style={{
            borderCollapse: "collapse",
            width: "100%",
            tableLayout: "fixed",
          }}
        >
          <thead>
            <tr>
              {sheetData[0]?.map((_, colIndex) => (
                <th
                  key={colIndex}
                  style={{
                    backgroundColor: "#f5f5f5",
                    border: "1px solid #ddd",
                    padding: "8px 6px",
                    textAlign: "left",
                    fontWeight: 600,
                    color: "#333",
                    minWidth: getColumnWidth(colIndex),
                    width: getColumnWidth(colIndex),
                    position: "sticky",
                    top: 0,
                    zIndex: 1,
                  }}
                >
                  {XLSX.utils.encode_col(colIndex)}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {sheetData.map((row, rowIndex) => (
              <tr key={rowIndex}>
                {(row as unknown[]).map((cell, colIndex) => (
                  <td
                    key={colIndex}
                    onDoubleClick={() => handleDoubleClick(rowIndex, colIndex)}
                    style={{
                      border: "1px solid #ddd",
                      padding: "6px",
                      minWidth: getColumnWidth(colIndex),
                      width: getColumnWidth(colIndex),
                      cursor: "cell",
                      backgroundColor:
                        editingCell?.row === rowIndex &&
                        editingCell?.col === colIndex
                          ? "#fff9c4"
                          : "#fff",
                    }}
                  >
                    {editingCell?.row === rowIndex &&
                    editingCell?.col === colIndex ? (
                      <input
                        ref={inputRef}
                        value={editingCell.value}
                        onChange={(e) =>
                          setEditingCell({
                            ...editingCell,
                            value: e.target.value,
                          })
                        }
                        onBlur={handleEditSave}
                        onKeyDown={handleKeyDown}
                        style={{
                          width: "100%",
                          border: "none",
                          outline: "none",
                          fontSize: 14,
                          fontFamily: "sans-serif",
                          backgroundColor: "transparent",
                          padding: 0,
                          margin: 0,
                        }}
                      />
                    ) : (
                      <span
                        style={{
                          display: "block",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                          whiteSpace: "nowrap",
                        }}
                      >
                        {cell !== undefined ? String(cell) : ""}
                      </span>
                    )}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
