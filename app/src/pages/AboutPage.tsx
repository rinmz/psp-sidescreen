import { SettingsGroup } from "../components/ui/SettingsGroup";

const CREDITS = [
  {
    name: "PSPdisp",
    what: "The program that runs on the PSP, and the way the PC talks to it.",
    licence: "BSD",
  },
  {
    name: "Virtual Display Driver",
    what: "The extra screen that Windows shows for the PSP.",
    licence: "MIT",
  },
  {
    name: "Handy",
    what: "The look of this window follows Handy's design.",
    licence: "MIT",
  },
] as const;

export function AboutPage({ version }: { version: string }) {
  return (
    <div className="mx-auto w-full max-w-3xl space-y-6">
      <section className="rounded-lg border border-line bg-background p-4">
        <h2 className="text-base font-semibold">PSP Sidescreen</h2>
        <p className="mt-1 text-sm text-ink-2">
          Use a PlayStation Portable as a second screen for your Windows PC. Version{" "}
          {version}.
        </p>
      </section>

      <SettingsGroup title="Built on">
        {CREDITS.map(({ name, what, licence }) => (
          <div key={name} className="flex items-center justify-between gap-4 p-2 px-4">
            <div>
              <h3 className="text-sm font-medium">{name}</h3>
              <p className="text-sm text-ink-2">{what}</p>
            </div>
            <span className="rounded-md border border-line px-2 py-0.5 text-xs text-ink-2">
              {licence}
            </span>
          </div>
        ))}
      </SettingsGroup>
    </div>
  );
}
