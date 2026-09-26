# `teistro-gochar`

Gochar: each transiting graha read from the natal Moon's sign, as
Phaladeepika ch. 26 teaches it (`03-design/gochar.md`).

What is built is the snapshot every later part reads: for each of the
nine grahas at an instant, its house from the reference sign, whether that
house is good (v. 2), the vedha house that can obstruct it (vv. 3 to 8),
**which grahas obstruct it** after the verses' exemptions, the verdict,
and whether it stands in the decanate where its transit bears fruit
(v. 25). The table is the text's, read on the printed page, and the two
places the text is silent — the nodes' vedha and whether the nodes
obstruct — are settings (`gochar.node_vedha`, `gochar.node_obstruction`).
