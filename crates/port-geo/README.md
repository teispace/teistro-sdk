# teistro-port-geo

The geo port of the Teistro SDK: the contract a geographic dataset
implements so a caller can resolve a place, its IANA zone, its ground
height (with the height's source, resolution and datum) and the ISO 3166
codes of the admin units holding it, and a small table provider
(`GeoTable`) for tests and for a consumer's own places. The SDK never
consults it implicitly; a resolved place travels in the request it is used
in, so a stored chart replays without it (`docs/03-design/geo-port.md`).
