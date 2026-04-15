import { useEffect } from "react";
import { useUIStore } from "@/stores/ui-store";
import { Sidebar } from "./sidebar";

export function AppShell({ children }: { children: React.ReactNode }) {
  const baseMap = useUIStore((s) => s.baseMap);

  useEffect(() => {
    const theme = baseMap === "light" ? "light" : "dark";
    document.documentElement.setAttribute("data-theme", theme);
  }, [baseMap]);

  return (
    <div className="relative h-screen w-screen overflow-hidden">
      <Sidebar />
      <main className="absolute inset-0">{children}</main>
    </div>
  );
}
