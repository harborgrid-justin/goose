import { useState } from "react";
import { Mic, ArrowUp } from "lucide-react";

interface GlobalComposerPillProps {
  onSend: (text: string) => void;
}

const PLACEHOLDER = "Start a conversation";

export function GlobalComposerPill({ onSend }: GlobalComposerPillProps) {
  const [text, setText] = useState("");

  const handleSend = () => {
    const trimmed = text.trim();
    if (!trimmed) return;
    onSend(trimmed);
    setText("");
  };

  return (
    <div
      className="fixed bottom-6 right-6 z-40 flex h-[68px] w-[482px] max-w-[calc(100vw-48px)] items-center gap-3 rounded-composer bg-[var(--surface-composer)] pl-[30px] pr-4"
      role="region"
      aria-label="Quick compose"
    >
      <input
        type="text"
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            handleSend();
          }
        }}
        placeholder={PLACEHOLDER}
        className="flex-1 appearance-none border-0 bg-transparent text-[16px] leading-[20px] text-black/70 outline-none placeholder:text-black/70 focus:outline-none focus:ring-0"
      />

      <button
        type="button"
        className="flex h-8 w-8 items-center justify-center rounded-full bg-[var(--surface-button)]"
        aria-label="Voice dictation"
        title="Voice dictation (coming soon)"
      >
        <Mic className="size-4 text-black/70" />
      </button>

      <button
        type="button"
        onClick={handleSend}
        className="flex h-8 w-10 items-center justify-center rounded-full bg-[var(--surface-button)]"
        aria-label="Send"
      >
        <ArrowUp className="size-4 text-black/70" />
      </button>
    </div>
  );
}
