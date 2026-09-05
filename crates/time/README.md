# teistro-time

The time layer of the Teistro SDK (`docs/03-design/time-and-timezone.md`).
This seed holds what the calendars already need: a zone's offset history
as rows in force from an instant, answering as a local clock, and the
shipped histories (Nepal's, from tzdb). Time scales and Delta T, zone
resolution with its replay-safe metadata, the sunrise-anchored local day
and ghati-pala follow.
