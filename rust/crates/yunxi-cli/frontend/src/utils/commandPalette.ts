export interface PaletteCommand {
  id: string;
  label: string;
  detail?: string;
  group: string;
  keywords?: string[];
  /** 选中后进入参数输入（用于 /search、/analyze） */
  needsInput?: boolean;
  slashTemplate?: string;
  run: () => void;
}

export function filterPaletteCommands(
  commands: PaletteCommand[],
  query: string,
): PaletteCommand[] {
  const q = query.trim().toLowerCase();
  if (!q) return commands;
  return commands.filter((cmd) => {
    const hay = [cmd.label, cmd.detail ?? '', cmd.group, ...(cmd.keywords ?? [])]
      .join(' ')
      .toLowerCase();
    return hay.includes(q);
  });
}
