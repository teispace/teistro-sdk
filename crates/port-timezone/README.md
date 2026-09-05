# teistro-port-timezone

The time-zone port of the Teistro SDK: the contract a zone database
implements so the time layer can place a civil time in a zone and read
the offset in force at an instant, with the facts a replay needs (the
abbreviation, whether the instant precedes the zone's first rule, which
offsets the zone applies today). The embedded database lives in
`teistro-time`; a consumer may supply another at context creation
(`docs/03-design/time-and-timezone.md`).
