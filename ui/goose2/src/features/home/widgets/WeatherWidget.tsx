import { useTranslation } from "react-i18next";
import { IconCloud, IconCloudRain, IconSun } from "@tabler/icons-react";

const FORECAST = [
  { key: "today", temp: "72", icon: IconSun },
  { key: "tomorrow", temp: "68", icon: IconCloud },
  { key: "friday", temp: "63", icon: IconCloudRain },
] as const;

export function WeatherWidget() {
  const { t } = useTranslation("home");

  return (
    <section className="flex h-full w-full flex-col rounded-lg border border-[#06B6D4]/20 bg-[#EAFBFF] p-4 text-[var(--text-default-alex)] backdrop-blur">
      <div className="flex items-start justify-between">
        <div>
          <p className="text-[13px] text-[var(--text-muted-alex)]">
            {t("widgets.weather.location")}
          </p>
          <p className="mt-1 text-[40px] font-light leading-none">72°</p>
        </div>
        <div className="rounded-full bg-[#FEF3C7] p-3 text-[#F59E0B]">
          <IconSun className="size-5" />
        </div>
      </div>

      <p className="mt-3 text-sm leading-5 text-[var(--text-muted-alex)]">
        {t("widgets.weather.summary")}
      </p>

      <div className="mt-auto grid grid-cols-3 gap-1.5 pt-3">
        {FORECAST.map(({ key, temp, icon: Icon }) => (
          <div
            key={key}
            className="flex min-h-[74px] flex-col items-center justify-center gap-1.5 rounded-md bg-[#06B6D4]/10 px-2 py-2 text-center"
          >
            <Icon className="size-3.5 text-[#0891B2]" />
            <span className="text-[11px] text-[var(--text-muted-alex)]">
              {t(`widgets.weather.forecast.${key}`)}
            </span>
            <span className="text-[13px] leading-none">{temp}°</span>
          </div>
        ))}
      </div>
    </section>
  );
}
