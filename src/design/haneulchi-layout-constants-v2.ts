/** Haneulchi Layout Constants v2
 * These values are locked to the 1536x864 multi-screen concepts and the
 * 1536x1024 terminal-deck reference. Do not change without design review.
 */

export const HC_VIEWPORTS = {
  primary: { width: 1536, height: 864 },
  terminalDeckLarge: { width: 1536, height: 1024 },
  minMvp: { width: 1280, height: 760 },
} as const;

export const HC_LAYOUT = {
  titlebarH: 40,
  statusbarH: 26,
  sidebarW: 264,
  rightRailW: 320,
  workspaceTabsH: 42,
  pageToolbarH: 44,
  paneGap: 8,
  panelGap: 8,
  contentPad: 8,
  sidebarPadX: 12,
  sidebarPadY: 8,
  rightRailPad: 8,
  bottomPanelH: 176,
  bottomPanelCompactH: 150,
  projectRowH: 56,
  sidebarSearchH: 34,
  quickActionsH: 160,
  terminalHeaderH: 34,
  panelHeaderH: 38,
  cardRadius: 8,
} as const;

export const HC_COMPUTED_PRIMARY = {
  contentH: HC_VIEWPORTS.primary.height - HC_LAYOUT.titlebarH - HC_LAYOUT.statusbarH,
  workspaceW: HC_VIEWPORTS.primary.width - HC_LAYOUT.sidebarW - HC_LAYOUT.rightRailW,
  workspaceBodyH:
    HC_VIEWPORTS.primary.height -
    HC_LAYOUT.titlebarH -
    HC_LAYOUT.statusbarH -
    HC_LAYOUT.workspaceTabsH,
} as const;

export const HC_SCREEN_REGIONS = {
  titlebar: { x: 0, y: 0, w: 1536, h: 40 },
  sidebar: { x: 0, y: 40, w: 264, h: 798 },
  workspace: { x: 264, y: 40, w: 952, h: 798 },
  rightRail: { x: 1216, y: 40, w: 320, h: 798 },
  statusbar: { x: 0, y: 838, w: 1536, h: 26 },
} as const;

export const HC_TYPE = {
  displaySm: { fontSize: 22, lineHeight: 28, weight: 700 },
  headingMd: { fontSize: 16, lineHeight: 22, weight: 650 },
  headingSm: { fontSize: 14, lineHeight: 20, weight: 650 },
  bodyMd: { fontSize: 13, lineHeight: 20, weight: 400 },
  bodySm: { fontSize: 12, lineHeight: 18, weight: 400 },
  labelMd: { fontSize: 12, lineHeight: 16, weight: 500 },
  labelSm: { fontSize: 11, lineHeight: 14, weight: 600 },
  labelXs: { fontSize: 10, lineHeight: 12, weight: 600 },
  terminalMd: { fontSize: 13, lineHeight: 19, weight: 400 },
  terminalSm: { fontSize: 12, lineHeight: 18, weight: 400 },
} as const;

export const HC_XTERM_THEME = {
  background: "#071116",
  foreground: "#D8E0E8",
  cursor: "#A8C7FF",
  cursorAccent: "#06111E",
  selectionBackground: "rgba(168, 199, 255, 0.24)",
  black: "#071116",
  red: "#FF6B6B",
  green: "#62D46E",
  yellow: "#F2B84B",
  blue: "#7DB7FF",
  magenta: "#C79BFF",
  cyan: "#68D7E8",
  white: "#D8E0E8",
  brightBlack: "#4B5561",
  brightRed: "#FF8787",
  brightGreen: "#7CF08A",
  brightYellow: "#FFD166",
  brightBlue: "#A8C7FF",
  brightMagenta: "#D8B4FF",
  brightCyan: "#8CEBFF",
  brightWhite: "#F1F5F9",
} as const;

export const HC_XTERM_OPTIONS = {
  fontFamily: "JetBrains Mono, SF Mono, ui-monospace, Menlo, Monaco, Consolas, monospace",
  fontSize: 13,
  lineHeight: 1.45,
  letterSpacing: 0,
  cursorBlink: true,
  cursorStyle: "block" as const,
  scrollback: 10000,
  smoothScrollDuration: 0,
  theme: HC_XTERM_THEME,
} as const;

export const HC_Z_INDEX = {
  base: 0,
  panel: 10,
  tabbar: 20,
  titlebar: 30,
  drawer: 40,
  modal: 50,
  commandPalette: 60,
  toast: 70,
} as const;
