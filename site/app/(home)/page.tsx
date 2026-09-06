import Link from 'next/link';
import { appName, tagline } from '@/lib/shared';

/** What a binding computes, shown as itself rather than described. */
const facts = [
  ['14 April 2015', '2072-01-01 BS', 'the calendars, with the era and how the date was resolved'],
  ['00:20, 1 January 1986, Kathmandu', 'JD 2446431.274306 UTC', '+05:45, from tzdb 2026c, with the rule that applied'],
  ['the Sun at J2000', '278.5768°', 'apparent geocentric ecliptic of date, from any provider'],
  ['sdk.reason.grahaInBhava', 'गुरु ७औं भावमा', 'the locale engine, Nepali-first'],
];

export default function HomePage() {
  return (
    <main className="flex flex-col flex-1 items-center px-4 py-16">
      <div className="w-full max-w-3xl">
        <h1 className="text-4xl font-bold tracking-tight">{appName}</h1>
        <p className="mt-4 text-lg text-fd-muted-foreground">{tagline}</p>

        <div className="mt-8 flex gap-3">
          <Link
            href="/docs"
            className="rounded-md bg-fd-primary px-4 py-2 text-sm font-medium text-fd-primary-foreground"
          >
            Read the docs
          </Link>
          <Link
            href="/docs/reference"
            className="rounded-md border px-4 py-2 text-sm font-medium"
          >
            The reference
          </Link>
        </div>

        <h2 className="mt-16 text-sm font-semibold uppercase tracking-wide text-fd-muted-foreground">
          The same answers from C, Node and Dart
        </h2>
        <dl className="mt-4 divide-y rounded-lg border">
          {facts.map(([asked, answered, note]) => (
            <div key={asked} className="grid gap-1 p-4 sm:grid-cols-[1fr_auto]">
              <dt className="font-mono text-sm">{asked}</dt>
              <dd className="font-mono text-sm font-medium sm:text-right">{answered}</dd>
              <p className="text-sm text-fd-muted-foreground sm:col-span-2">{note}</p>
            </div>
          ))}
        </dl>

        <p className="mt-8 text-sm text-fd-muted-foreground">
          Every number carries where it came from: the ephemeris that produced it, the settings
          hash that fixes the frame, and the model each correction used. Apache-2.0, on{' '}
          <Link
            href="https://github.com/teispace/teistro-sdk"
            className="font-medium underline"
          >
            GitHub
          </Link>
          .
        </p>
      </div>
    </main>
  );
}
