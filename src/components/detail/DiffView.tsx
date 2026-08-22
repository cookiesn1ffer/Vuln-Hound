import { cn } from "@/lib/utils";

export function DiffView({ diff }: { diff: string }) {
  const lines = diff.split("\n");

  return (
    <div className="overflow-hidden rounded-md border border-border font-mono text-[12px] leading-5">
      {lines.map((line, i) => {
        const kind = line.startsWith("+")
          ? "add"
          : line.startsWith("-")
            ? "remove"
            : "context";
        return (
          <div
            key={i}
            className={cn(
              "px-3 whitespace-pre-wrap",
              kind === "add" && "bg-severity-low/10 text-foreground",
              kind === "remove" && "bg-severity-critical/10 text-foreground",
              kind === "context" && "text-muted-foreground"
            )}
          >
            {line || " "}
          </div>
        );
      })}
    </div>
  );
}
