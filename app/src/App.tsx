import { useState } from "react";

import "./App.css";
import { Footer } from "./components/Footer";
import { Sidebar, type Section } from "./components/Sidebar";
import { useSidescreen } from "./hooks/useSidescreen";
import { AboutPage } from "./pages/AboutPage";
import { ControlsPage } from "./pages/ControlsPage";
import { DisplayPage } from "./pages/DisplayPage";
import { SetupPage } from "./pages/SetupPage";

export default function App() {
  const [chosen, setSection] = useState<Section | null>(null);
  const sidescreen = useSidescreen();
  // Until the person has been through the Setup page it is the first thing they see.
  const section: Section =
    chosen ?? (sidescreen.settings.setupDone ? "display" : "setup");
  const finishSetup = () => {
    sidescreen.update({ setupDone: true });
    setSection("display");
  };

  return (
    <div className="flex h-screen cursor-default select-none flex-col">
      <div className="flex flex-1 overflow-hidden">
        <Sidebar active={section} onChange={setSection} />
        <main className="flex-1 overflow-y-auto p-4">
          {sidescreen.ready && section === "setup" && (
            <SetupPage sidescreen={sidescreen} onFinish={finishSetup} />
          )}
          {sidescreen.ready && section === "display" && (
            <DisplayPage sidescreen={sidescreen} />
          )}
          {sidescreen.ready && section === "controls" && (
            <ControlsPage sidescreen={sidescreen} />
          )}
          {sidescreen.ready && section === "about" && (
            <AboutPage version={sidescreen.version} />
          )}
        </main>
      </div>
      <Footer state={sidescreen.state} version={sidescreen.version} />
    </div>
  );
}
