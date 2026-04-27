import { useState } from "react";
import { useTranslation } from "react-i18next";
import { MoreVertical, Copy, Pencil, Trash2, Download } from "lucide-react";
import figureUrl from "@/assets/agents/figure.png";
import { cn } from "@/shared/lib/cn";
import { Button } from "@/shared/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/shared/ui/dropdown-menu";
import type { Persona } from "@/shared/types/agents";
import { getPersonaSource } from "@/features/agents/lib/personaPresentation";

interface PersonaCardProps {
  persona: Persona;
  onSelect?: (persona: Persona) => void;
  onEdit?: (persona: Persona) => void;
  onDuplicate?: (persona: Persona) => void;
  onDelete?: (persona: Persona) => void;
  onExport?: (persona: Persona) => void;
  isActive?: boolean;
}

export function PersonaCard({
  persona,
  onSelect,
  onEdit,
  onDuplicate,
  onDelete,
  onExport,
  isActive = false,
}: PersonaCardProps) {
  const { t } = useTranslation(["agents", "common"]);
  const [menuOpen, setMenuOpen] = useState(false);

  const personaSource = getPersonaSource(persona);
  const canEditPersona = personaSource === "custom";
  const canDeletePersona = personaSource !== "builtin";

  const handleCardKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if (event.target !== event.currentTarget || menuOpen) {
      return;
    }
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onSelect?.(persona);
    }
  };

  return (
    // biome-ignore lint/a11y/useSemanticElements: card contains nested menu buttons, so a native button is not valid here
    <div
      aria-label={t("card.ariaLabel", { name: persona.displayName })}
      role="button"
      onClick={() => !menuOpen && onSelect?.(persona)}
      onKeyDown={handleCardKeyDown}
      tabIndex={0}
      className={cn(
        "group relative flex flex-col items-center cursor-pointer px-3 py-4",
        "transition-colors duration-200",
        isActive && "bg-black/[0.03]",
      )}
    >
      {/* Single shared cutout figure — visual placeholder; real per-persona avatars deferred */}
      <img
        src={figureUrl}
        alt=""
        aria-hidden="true"
        className="h-[220px] w-auto select-none"
      />

      <div className="mt-3 h-px w-[149px] bg-black/30" />

      <span className="mt-3 inline-flex h-5 items-center rounded-full bg-[var(--surface-button)] px-[6px] pb-[3px] text-[14px] text-[var(--text-title-alex)]">
        {persona.displayName}
      </span>

      <p className="mt-3 line-clamp-6 w-[149px] text-[16px] leading-[20px] text-[var(--text-muted-alex)]">
        {persona.systemPrompt}
      </p>

      <div className="absolute top-2 right-2 opacity-0 transition-opacity duration-150 focus-within:opacity-100 group-hover:opacity-100">
        <DropdownMenu open={menuOpen} onOpenChange={setMenuOpen}>
          <DropdownMenuTrigger asChild>
            <Button
              type="button"
              variant="ghost"
              size="icon-xs"
              aria-label={t("card.options")}
              onClick={(e) => e.stopPropagation()}
              onKeyDown={(event) => event.stopPropagation()}
              className="size-6 rounded-md text-muted-foreground hover:text-foreground"
            >
              <MoreVertical className="size-3.5" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" sideOffset={4}>
            {canEditPersona && (
              <DropdownMenuItem onSelect={() => onEdit?.(persona)}>
                <Pencil className="size-3.5" />
                {t("common:actions.edit")}
              </DropdownMenuItem>
            )}
            <DropdownMenuItem onSelect={() => onDuplicate?.(persona)}>
              <Copy className="size-3.5" />
              {t("common:actions.duplicate")}
            </DropdownMenuItem>
            {onExport && (
              <DropdownMenuItem onSelect={() => onExport(persona)}>
                <Download className="size-3.5" />
                {t("common:actions.export")}
              </DropdownMenuItem>
            )}
            {canDeletePersona && (
              <DropdownMenuItem
                variant="destructive"
                onSelect={() => onDelete?.(persona)}
              >
                <Trash2 className="size-3.5" />
                {t("common:actions.delete")}
              </DropdownMenuItem>
            )}
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  );
}
