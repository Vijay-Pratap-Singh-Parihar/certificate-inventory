"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Home, Archive } from "lucide-react";

export function Sidebar({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}) {
  const pathname = usePathname();
  const navItems: { href: string; label: string; icon: React.ReactNode }[] = [
    { href: "/", label: "Dashboard", icon: <Home className="w-5 h-5" /> },
    { href: "/inventory", label: "Inventory", icon: <Archive className="w-5 h-5" /> },
  ];

  return (
    <>
      <div
        id="mobile-overlay"
        className={"fixed inset-0 bg-gray-900 bg-opacity-50 z-20 lg:hidden " + (open ? "" : "hidden")}
        onClick={onClose}
        aria-hidden
      />
      <aside
        id="sidebar"
        className={
          "bg-slate-900 text-white w-64 flex-shrink-0 flex flex-col transition-transform transform absolute lg:relative z-30 h-full " +
          (open ? "translate-x-0" : "-translate-x-full lg:translate-x-0")
        }
      >
        <div className="h-16 flex items-center px-6 border-b border-slate-700 font-bold text-xl tracking-wider">
          🛡️ CertManager
        </div>
        <nav className="flex-1 py-6 px-3 space-y-2">
          {navItems.map(({ href, label, icon }) => {
            const isActive = href !== "#" && pathname === href;
            return (
              <Link
                key={href + label}
                href={href}
                className={
                  "flex items-center px-3 py-2.5 rounded-lg transition-colors " +
                  (isActive
                    ? "bg-indigo-600 text-white"
                    : "text-slate-300 hover:text-white hover:bg-slate-800")
                }
              >
                <span className="mr-3">{icon}</span>
                {label}
              </Link>
            );
          })}
        </nav>
      </aside>
    </>
  );
}
