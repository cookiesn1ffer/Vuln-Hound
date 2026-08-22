import type { ReactNode } from "react";

export function AppShell({
  sidebar,
  main,
  detail,
}: {
  sidebar: ReactNode;
  main: ReactNode;
  detail: ReactNode | null;
}) {
  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
      <aside className="flex w-[272px] shrink-0 flex-col border-r border-border bg-sidebar">
        {sidebar}
      </aside>
      <main className="flex min-w-0 flex-1 flex-col overflow-hidden">{main}</main>
      {detail && (
        <aside className="flex w-[420px] shrink-0 flex-col overflow-hidden border-l border-border bg-card">
          {detail}
        </aside>
      )}
    </div>
  );
}
