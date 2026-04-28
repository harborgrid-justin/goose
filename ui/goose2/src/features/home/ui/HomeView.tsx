import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useLocaleFormatting } from "@/shared/i18n";
import worldCubeUrl from "@/assets/home/world-cube.png";
import clockUrl from "@/assets/home/clock.svg";
import person2Url from "@/assets/home/person-2.png";
import stickyNoteUrl from "@/assets/home/sticky-note.svg";

function getGreetingKey(hour: number): "morning" | "afternoon" | "evening" {
  if (hour < 12) return "morning";
  if (hour < 17) return "afternoon";
  return "evening";
}

// HomeClock — editorial-display typography. fontFamily is set inline so we
// don't depend on inheritance from <body>; font-light (weight 300) targets
// the Cash Sans Light @font-face declared in globals.css.
function HomeClock() {
  const [time, setTime] = useState(new Date());
  const { getTimeParts } = useLocaleFormatting();

  useEffect(() => {
    const interval = setInterval(() => setTime(new Date()), 1000);
    return () => clearInterval(interval);
  }, []);

  const { hour, minute, dayPeriod } = getTimeParts(time, {
    hour: "numeric",
    minute: "2-digit",
  });

  return (
    <div
      className="flex items-baseline gap-3"
      style={{ fontFamily: "var(--font-sans-alex)" }}
    >
      <span className="text-[120px] font-light leading-none tracking-[-0.04em] text-foreground">
        {hour}:{minute}
      </span>
      {dayPeriod ? (
        <span className="text-[40px] font-light leading-none tracking-[-0.02em] text-foreground">
          {dayPeriod}
        </span>
      ) : null}
    </div>
  );
}

export function HomeView() {
  const { t } = useTranslation("home");
  const [hour] = useState(() => new Date().getHours());
  const greeting = t(`greeting.${getGreetingKey(hour)}`);

  return (
    <div className="relative h-full w-full overflow-hidden">
      {/* Time + greeting — primary focal element, vertically centered */}
      <div className="pointer-events-none absolute left-[8%] top-[42%] flex -translate-y-1/2 flex-col gap-3">
        <HomeClock />
        <p
          className="text-[28px] font-light leading-tight tracking-[-0.02em] text-foreground/70"
          style={{ fontFamily: "var(--font-sans-alex)" }}
        >
          {greeting},{" "}
          {/* i18n-check-ignore: placeholder for dynamic user name — will be replaced when user profile lookup ships */}
          <span>Tulsi</span>.
        </p>
      </div>

      {/* Cube — right-center, gives the right side its visual weight */}
      <img
        src={worldCubeUrl}
        alt=""
        aria-hidden="true"
        className="pointer-events-none absolute right-[14%] top-1/2 w-[30%] max-w-[560px] -translate-y-1/2 select-none"
      />

      {/* Clock — top right corner */}
      <img
        src={clockUrl}
        alt=""
        aria-hidden="true"
        className="pointer-events-none absolute right-[6%] top-[6%] w-[11%] max-w-[180px] select-none"
      />

      {/* Person2 — pushed to the bottom-right corner so cube has breathing room */}
      <img
        src={person2Url}
        alt=""
        aria-hidden="true"
        className="pointer-events-none absolute right-[5%] bottom-[8%] w-[7%] max-w-[120px] select-none"
      />

      {/* Sticky note — pulled rightward to ~center-bottom so it anchors the
          lower band rather than crowding the bottom-left corner alone */}
      <img
        src={stickyNoteUrl}
        alt=""
        aria-hidden="true"
        className="pointer-events-none absolute bottom-[10%] left-[28%] w-[15%] max-w-[260px] select-none"
      />
    </div>
  );
}
