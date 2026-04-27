import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { cn } from "@/shared/lib/cn";
import { Skeleton } from "@/shared/ui/skeleton";
import type { Persona } from "@/shared/types/agents";
import { PersonaCard } from "@/features/agents/ui/PersonaCard";

interface PersonaGalleryProps {
  personas: Persona[];
  activePersonaId?: string;
  onSelectPersona: (persona: Persona) => void;
  onEditPersona: (persona: Persona) => void;
  onDuplicatePersona: (persona: Persona) => void;
  onDeletePersona: (persona: Persona) => void;
  onExportPersona?: (persona: Persona) => void;
  isLoading?: boolean;
  dropHandlers?: React.HTMLAttributes<HTMLElement>;
  isDragOver?: boolean;
}

function SkeletonCard() {
  return (
    <div aria-hidden="true" className="flex flex-col items-center px-3 py-4">
      <Skeleton className="h-[220px] w-[110px]" />
      <Skeleton className="mt-3 h-px w-[149px]" />
      <Skeleton className="mt-3 h-5 w-20" />
      <Skeleton className="mt-3 h-4 w-full" />
      <Skeleton className="mt-1 h-4 w-3/4" />
    </div>
  );
}

export function PersonaGallery({
  personas,
  activePersonaId,
  onSelectPersona,
  onEditPersona,
  onDuplicatePersona,
  onDeletePersona,
  onExportPersona,
  isLoading = false,
  dropHandlers,
  isDragOver = false,
}: PersonaGalleryProps) {
  const { t } = useTranslation("agents");
  const sorted = useMemo(() => {
    const builtins = personas
      .filter((p) => p.isBuiltin)
      .sort((a, b) => a.displayName.localeCompare(b.displayName));
    const custom = personas
      .filter((p) => !p.isBuiltin)
      .sort((a, b) => a.displayName.localeCompare(b.displayName));
    return [...builtins, ...custom];
  }, [personas]);

  if (isLoading) {
    return (
      <section
        role="status"
        aria-label={t("gallery.loading")}
        className="grid grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-x-12 gap-y-16 p-8"
      >
        <SkeletonCard />
        <SkeletonCard />
        <SkeletonCard />
        <SkeletonCard />
      </section>
    );
  }

  return (
    <section
      {...dropHandlers}
      className={cn(
        "grid grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-x-12 gap-y-16 rounded-tile p-8 transition-colors",
        isDragOver && "ring-2 ring-ring ring-offset-2",
      )}
    >
      {sorted.map((persona) => (
        <PersonaCard
          key={persona.id}
          persona={persona}
          isActive={persona.id === activePersonaId}
          onSelect={onSelectPersona}
          onEdit={onEditPersona}
          onDuplicate={onDuplicatePersona}
          onDelete={onDeletePersona}
          onExport={onExportPersona}
        />
      ))}
    </section>
  );
}
