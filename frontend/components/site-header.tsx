import Link from "next/link";
import { CalendarDays } from "lucide-react";

export function SiteHeader() {
  return (
    <header className="border-b border-border/60">
      <div className="mx-auto flex h-14 w-full max-w-6xl items-center justify-between px-6">
        <Link
          href="/"
          className="flex items-center gap-2 text-sm font-semibold tracking-tight"
        >
          <CalendarDays className="size-4 text-primary" />
          Calendar
        </Link>
        <nav className="flex items-center gap-6 text-sm text-muted-foreground">
          <Link
            href="/booking"
            className="transition-colors hover:text-foreground"
          >
            Записаться
          </Link>
          <Link
            href="/admin"
            className="transition-colors hover:text-foreground"
          >
            Админка
          </Link>
        </nav>
      </div>
    </header>
  );
}
