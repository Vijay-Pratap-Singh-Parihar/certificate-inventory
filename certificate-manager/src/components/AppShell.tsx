"use client";

import { useState } from "react";
import { Menu } from "lucide-react";
import { Sidebar } from "./Sidebar";

export function AppShell({
  children,
  breadcrumb,
  action,
}: {
  children: React.ReactNode;
  breadcrumb: React.ReactNode;
  action?: React.ReactNode;
}) {
  const [sidebarOpen, setSidebarOpen] = useState(false);

  return (
    <div className="bg-gray-50 text-gray-800 font-sans antialiased flex h-screen overflow-hidden">
      <Sidebar open={sidebarOpen} onClose={() => setSidebarOpen(false)} />
      <main className="flex-1 flex flex-col h-full overflow-hidden w-full relative">
        <header className="h-16 bg-white border-b border-gray-200 flex items-center justify-between px-4 sm:px-6 lg:px-8 flex-shrink-0">
          <div className="flex items-center">
            <button
              type="button"
              onClick={() => setSidebarOpen(true)}
              className="text-gray-500 hover:text-gray-700 focus:outline-none lg:hidden mr-4"
              aria-label="Open menu"
            >
              <Menu className="h-6 w-6" />
            </button>
            <div className="text-sm text-gray-500 font-medium">{breadcrumb}</div>
          </div>
          {action}
        </header>
        {children}
      </main>
    </div>
  );
}
