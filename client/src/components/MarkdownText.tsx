import { Fragment, type ReactNode } from "react";

interface MarkdownTextProps {
  content: string;
  className?: string;
  mode?: "full" | "lists";
}

function parseInline(text: string): ReactNode[] {
  const nodes: ReactNode[] = [];
  const pattern = /(\*\*[^*]+\*\*|`[^`]+`|\[[^\]]+\]\((https?:\/\/[^\s)]+)\))/g;
  let last = 0;
  let match: RegExpExecArray | null;

  while ((match = pattern.exec(text)) !== null) {
    if (match.index > last) {
      nodes.push(text.slice(last, match.index));
    }

    const token = match[0];
    if (token.startsWith("**") && token.endsWith("**")) {
      nodes.push(<strong key={`${match.index}-bold`}>{token.slice(2, -2)}</strong>);
    } else if (token.startsWith("`") && token.endsWith("`")) {
      nodes.push(
        <code
          key={`${match.index}-code`}
          className="rounded bg-white/10 px-1 py-0.5 font-mono text-[0.9em]"
        >
          {token.slice(1, -1)}
        </code>,
      );
    } else {
      const link = token.match(/^\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)$/);
      if (link) {
        nodes.push(
          <a
            key={`${match.index}-link`}
            href={link[2]}
            target="_blank"
            rel="noreferrer noopener"
            className="underline decoration-white/40 underline-offset-2 hover:decoration-white/80"
          >
            {link[1]}
          </a>,
        );
      } else {
        nodes.push(token);
      }
    }

    last = pattern.lastIndex;
  }

  if (last < text.length) {
    nodes.push(text.slice(last));
  }

  return nodes;
}

export default function MarkdownText({ content, className = "", mode = "full" }: MarkdownTextProps) {
  const lines = content.split("\n");
  const blocks: ReactNode[] = [];
  let i = 0;
  let key = 0;

  while (i < lines.length) {
    const line = lines[i];
    if (!line.trim()) {
      i += 1;
      continue;
    }

    if (mode === "full" && line.startsWith("```")) {
      i += 1;
      const codeLines: string[] = [];
      while (i < lines.length && !lines[i].startsWith("```")) {
        codeLines.push(lines[i]);
        i += 1;
      }
      if (i < lines.length) {
        i += 1;
      }
      blocks.push(
        <pre
          key={`block-${key++}`}
          className="my-2 overflow-x-auto rounded-lg border border-white/10 bg-black/30 p-3"
        >
          <code>{codeLines.join("\n")}</code>
        </pre>,
      );
      continue;
    }

    const heading = line.match(/^(#{1,3})\s+(.+)$/);
    if (mode === "full" && heading) {
      const level = heading[1].length;
      const text = heading[2];
      const headingClass =
        level === 1 ? "text-xl font-semibold" : level === 2 ? "text-lg font-semibold" : "text-base font-semibold";
      blocks.push(
        <p key={`block-${key++}`} className={`my-1 ${headingClass}`}>
          {parseInline(text)}
        </p>,
      );
      i += 1;
      continue;
    }

    if (/^[-*]\s+/.test(line)) {
      const items: string[] = [];
      while (i < lines.length && /^[-*]\s+/.test(lines[i])) {
        items.push(lines[i].replace(/^[-*]\s+/, ""));
        i += 1;
      }
      blocks.push(
        <ul key={`block-${key++}`} className="my-2 list-disc space-y-1 pl-5">
          {items.map((item, idx) => (
            <li key={`li-${idx}`}>{parseInline(item)}</li>
          ))}
        </ul>,
      );
      continue;
    }

    if (/^\d+\.\s+/.test(line)) {
      const items: string[] = [];
      while (i < lines.length && /^\d+\.\s+/.test(lines[i])) {
        items.push(lines[i].replace(/^\d+\.\s+/, ""));
        i += 1;
      }
      blocks.push(
        <ol key={`block-${key++}`} className="my-2 list-decimal space-y-1 pl-5">
          {items.map((item, idx) => (
            <li key={`li-ol-${idx}`}>{mode === "full" ? parseInline(item) : item}</li>
          ))}
        </ol>,
      );
      continue;
    }

    const paragraph: string[] = [];
    while (
      i < lines.length &&
      lines[i].trim() &&
      (mode !== "full" || !lines[i].startsWith("```"))
    ) {
      if (
        (mode === "full" && /^(#{1,3})\s+/.test(lines[i])) ||
        /^[-*]\s+/.test(lines[i]) ||
        /^\d+\.\s+/.test(lines[i])
      ) {
        break;
      }
      paragraph.push(lines[i]);
      i += 1;
    }

    blocks.push(
      <p key={`block-${key++}`} className="my-1 whitespace-pre-wrap">
        {paragraph.map((pLine, idx) => (
          <Fragment key={`p-${idx}`}>
            {idx > 0 && <br />}
            {mode === "full" ? parseInline(pLine) : pLine}
          </Fragment>
        ))}
      </p>,
    );
  }

  return <div className={className}>{blocks}</div>;
}
