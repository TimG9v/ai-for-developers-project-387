import Link from "next/link";
import { ArrowRight } from "lucide-react";

import { Button } from "@/components/ui/button";

export default function Home() {
  return (
    <main className="mx-auto flex w-full max-w-6xl flex-1 flex-col gap-12 px-6 py-16 lg:flex-row lg:items-start lg:justify-between lg:gap-16 lg:py-24">
      <section className="flex max-w-xl flex-col items-start gap-6">
        <span className="rounded-full border border-primary/40 bg-primary/10 px-3 py-1 text-xs font-medium tracking-wide text-primary uppercase">
          Быстрая запись на звонок
        </span>
        <h1 className="text-5xl font-bold tracking-tight">Calendar</h1>
        <p className="text-lg text-muted-foreground">
          Выберите свободный слот и запишитесь на 30-минутный звонок — без
          регистрации.
        </p>
        <Button
          asChild
          size="lg"
          className="px-5 text-base shadow-lg shadow-primary/30"
        >
          <Link href="/booking">
            Записаться
            <ArrowRight data-icon="inline-end" />
          </Link>
        </Button>
      </section>
      <aside className="w-full max-w-md rounded-2xl border border-border/60 bg-card p-8">
        <h2 className="text-xl font-semibold">Возможности</h2>
        <ul className="mt-4 flex flex-col gap-3 text-sm text-muted-foreground">
          <li>Выбор свободного слота на странице записи.</li>
          <li>Запись на 30-минутный звонок без регистрации.</li>
          <li>Список предстоящих встреч у владельца календаря.</li>
        </ul>
      </aside>
    </main>
  );
}
