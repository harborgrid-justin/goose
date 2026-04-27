import { render, screen, fireEvent } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { GlobalComposerPill } from "./GlobalComposerPill";

describe("GlobalComposerPill", () => {
  it("renders the universal 'Start a conversation' placeholder", () => {
    render(<GlobalComposerPill onSend={vi.fn()} />);
    expect(
      screen.getByPlaceholderText(/start a conversation/i),
    ).toBeInTheDocument();
  });

  it("calls onSend with the typed text when send is clicked", () => {
    const onSend = vi.fn();
    render(<GlobalComposerPill onSend={onSend} />);

    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "hello" } });
    fireEvent.click(screen.getByRole("button", { name: /send/i }));

    expect(onSend).toHaveBeenCalledWith("hello");
  });

  it("does not send when input is empty", () => {
    const onSend = vi.fn();
    render(<GlobalComposerPill onSend={onSend} />);

    fireEvent.click(screen.getByRole("button", { name: /send/i }));

    expect(onSend).not.toHaveBeenCalled();
  });
});
