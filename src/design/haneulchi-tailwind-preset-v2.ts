type HaneulchiTailwindPresetV2 = {
  darkMode: "class";
  theme: {
    extend: Record<string, unknown>;
  };
};

/**
 * Haneulchi Tailwind Preset v2.
 * Keep these aliases locked to the supplied dark terminal-first product-shell concepts.
 */
export const haneulchiTailwindPresetV2 = {
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        hc: {
          bg: {
            app: "var(--hc-bg-app)",
            chrome: "var(--hc-bg-chrome)",
            sidebar: "var(--hc-bg-sidebar)",
            workspace: "var(--hc-bg-workspace)",
            rightRail: "var(--hc-bg-right-rail)",
            statusbar: "var(--hc-bg-statusbar)",
          },
          surface: {
            sunken: "var(--hc-surface-sunken)",
            terminal: "var(--hc-surface-terminal)",
            terminalAlt: "var(--hc-surface-terminal-alt)",
            panel: "var(--hc-surface-panel)",
            panelLow: "var(--hc-surface-panel-low)",
            panelHigh: "var(--hc-surface-panel-high)",
            panelHighest: "var(--hc-surface-panel-highest)",
            card: "var(--hc-surface-card)",
            cardHover: "var(--hc-surface-card-hover)",
            glass: "var(--hc-surface-glass)",
            toolbar: "var(--hc-surface-toolbar)",
          },
          text: {
            primary: "var(--hc-text-primary)",
            secondary: "var(--hc-text-secondary)",
            muted: "var(--hc-text-muted)",
            faint: "var(--hc-text-faint)",
            disabled: "var(--hc-text-disabled)",
            inverse: "var(--hc-text-inverse)",
            terminal: "var(--hc-text-terminal)",
            terminalDim: "var(--hc-text-terminal-dim)",
          },
          accent: {
            primary: "var(--hc-accent-primary)",
            primaryStrong: "var(--hc-accent-primary-strong)",
            info: "var(--hc-accent-info)",
            success: "var(--hc-accent-success)",
            warning: "var(--hc-accent-warning)",
            danger: "var(--hc-accent-danger)",
            purple: "var(--hc-accent-purple)",
            pink: "var(--hc-accent-pink)",
            cyan: "var(--hc-accent-cyan)",
          },
          git: {
            added: "var(--hc-git-added)",
            modified: "var(--hc-git-modified)",
            deleted: "var(--hc-git-deleted)",
            renamed: "var(--hc-git-renamed)",
            untracked: "var(--hc-git-untracked)",
            main: "var(--hc-git-branch-main)",
            remote: "var(--hc-git-branch-remote)",
          },
          border: {
            hairline: "var(--hc-border-hairline)",
            ghost: "var(--hc-border-ghost)",
            subtle: "var(--hc-border-subtle)",
            strong: "var(--hc-border-strong)",
            accent: "var(--hc-border-accent)",
            success: "var(--hc-border-success)",
            warning: "var(--hc-border-warning)",
            danger: "var(--hc-border-danger)",
          },
        },
      },
      fontFamily: {
        ui: "var(--hc-font-ui)",
        display: "var(--hc-font-display)",
        label: "var(--hc-font-label)",
        mono: "var(--hc-font-mono)",
      },
      fontSize: {
        "hc-display-sm": ["22px", { lineHeight: "28px", fontWeight: "700" }],
        "hc-heading-md": ["16px", { lineHeight: "22px", fontWeight: "650" }],
        "hc-heading-sm": ["14px", { lineHeight: "20px", fontWeight: "650" }],
        "hc-body-md": ["13px", { lineHeight: "20px", fontWeight: "400" }],
        "hc-body-sm": ["12px", { lineHeight: "18px", fontWeight: "400" }],
        "hc-label-md": ["12px", { lineHeight: "16px", fontWeight: "500" }],
        "hc-label-sm": ["11px", { lineHeight: "14px", fontWeight: "600" }],
        "hc-label-xs": ["10px", { lineHeight: "12px", fontWeight: "600" }],
        "hc-terminal-md": ["13px", { lineHeight: "19px", fontWeight: "400" }],
        "hc-terminal-sm": ["12px", { lineHeight: "18px", fontWeight: "400" }],
      },
      spacing: {
        "hc-titlebar": "var(--hc-titlebar-h)",
        "hc-statusbar": "var(--hc-statusbar-h)",
        "hc-sidebar": "var(--hc-sidebar-w)",
        "hc-right-rail": "var(--hc-right-rail-w)",
        "hc-tabbar": "var(--hc-workspace-tabs-h)",
        "hc-toolbar": "var(--hc-page-toolbar-h)",
        "hc-bottom-panel": "var(--hc-bottom-panel-h)",
      },
      borderRadius: {
        "hc-xs": "var(--hc-radius-xs)",
        "hc-sm": "var(--hc-radius-sm)",
        "hc-md": "var(--hc-radius-md)",
        "hc-lg": "var(--hc-radius-lg)",
        "hc-xl": "var(--hc-radius-xl)",
        "hc-2xl": "var(--hc-radius-2xl)",
        "hc-pill": "var(--hc-radius-pill)",
      },
      boxShadow: {
        "hc-float": "var(--hc-shadow-float)",
        "hc-menu": "var(--hc-shadow-menu)",
        "hc-inner-pane": "var(--hc-shadow-inner-pane)",
        "hc-focus": "var(--hc-shadow-focus)",
        "hc-active-pane": "var(--hc-shadow-active-pane)",
      },
      backgroundImage: {
        "hc-primary": "var(--hc-gradient-primary)",
        "hc-panel-sheen": "var(--hc-gradient-panel-sheen)",
        "hc-terminal-vignette": "var(--hc-gradient-terminal-vignette)",
        "hc-window-vignette": "var(--hc-gradient-window-vignette)",
      },
      transitionTimingFunction: {
        "hc-standard": "cubic-bezier(.2,0,0,1)",
        "hc-pop": "cubic-bezier(.16,1,.3,1)",
      },
    },
  },
} satisfies HaneulchiTailwindPresetV2;

export default haneulchiTailwindPresetV2;
