# Ashauch: the period of impurity after a birth or a death

Status: `built`, 2026-10-22. Research and design for P2-3's second half
(the sankalpa half is `sankalpa.md`). §§1–3 are the research, §4.2 what
was built and where it differs from the draft, §5 the cases the tests
hold (T1–T29 on synthetic days in `crates/panchanga/tests/ashauch.rs`,
K1–K10 at Kathmandu in `crates/sdk/tests/ashauch.rs`). Every default is a
proposal the maintainer delegated, and each ships as data.

*Āśauca* (ashauch; Nepali *jutho* for a death, *sūtaka* for a birth) is the
span after a birth (*janana-*, *sūtaka*) or a death (*mṛta-*, *śāva*,
*mṛtaka*) during which a family is barred from rites and, for the closest,
from touch. An almanac app is asked three things: **from when, until when,
and why** (which clause of which text decided it). Every input the texts use
is a fact about the event, the deceased's rites, the relation, and when the
news arrived. The one input that is a caste in the texts (the length of the
full period, 10, 12, 15 or 30 days) becomes a plain number the family gives
(§3.1, §8).

## 1. Sources

| tag | work | rank | edition read | where it lives |
|---|---|---|---|---|
| Yā | *Yājñavalkya-smṛti* III.18–25 (*āśauca-prakaraṇa*) | 1 | GRETIL e-text of N. R. Acharya's 1949 Bombay edition (verse numbers `YajñS_3.n`) | GRETIL |
| Mit | Vijñāneśvara's *Mitākṣarā* on Yā III.18–25 | 1 | GRETIL/SARIT e-text (Davis, Olivelle et al. 2020, from Acharya 1949, Pandeya 1967, Setlur 1912), **CC BY-SA 4.0**: cite and quote the Sanskrit in docs, put none of the e-text into code or data | GRETIL, the āśauca-prakaraṇa on III.18–25 |
| Manu | *Manusmṛti* V.57–104 | 1 | GRETIL e-text; its numbers match the Kullūka vulgate (Bühler) for the verses cited here, and it marks a second numbering one lower ("[= M 5.n]") from 5.61 on | GRETIL |
| DS | *Dharmasindhu*, third pariccheda, uttarārdha, §§35–43 (*āśauca-nirṇaya*) | 1 | 1888 Nirnaya-sagara edition (`in.ernet.dli.2015.513461`), pp. 298–313; the editor's tabular *Āśauca-krama* at the end of the book, pp. 354–359 | the archive's OCR text; **pages from the running headers** (§1.1) |
| NS | *Nirṇayasindhu*, third pariccheda, uttarārdha, *āśauca-prakaraṇa* | 1 | Nirnaya-sagara 4th edition (ed. Narayan Ram Acharya), pp. 361–391 | an OCR text of that edition (headers 361, 363, 364 … 391 read) |
| Dk | NS with Veni Madhav Dhakal's Nepali *Cūḍāmaṇi* (Kathmandu 2071 BS) | 1 for NS's text, 2 for Dhakal's Nepali | archive.org `ryae-nirnaya-sindhu-chudamani-veni-madhav-dhakal-kamala`, OCR search text; Dhakal numbers this as the **fifth** pariccheda, pp. ~1060–1150 | the archive's OCR search text, pages from its page map |
| GP | *Garuḍa-purāṇa*, uttarakhaṇḍa (*pretakalpa*) 2.5, 2.34, 2.39; pūrvakhaṇḍa 1.106–107 (Yā paraphrased) | 1 | GRETIL e-text of the Venkatesvara Steam Press edition, **CC BY-NC-SA**: cite and quote briefly only | GRETIL |
| Kane | P. V. Kane, *History of Dharmaśāstra* vol. IV (1953), ch. VIII *Śuddhi*, pp. 267–333 | 2 | first edition, archive.org `HistoryOfDharmasastraancientAndMediaevalReligiousAndCivilLawV.4` | **cite the printed header**, which is one less than the archive's page map here (leaf 313's header reads 276, the map says 277) |
| Pr | "धर्मशास्त्रमा गरिएको मरणाशौचको सूक्ष्म चिन्तन", *Prashasan*, 14 Poush 2075 (29 Dec 2018), 38 numbered rules; credited in its tags to the astrologer Bhim Prasad Guragain | 3 | web | prasashan.com/2018/12/29/97760; republished by *Gaun Sahar*, 26 Kartik 2077 (11 Nov 2020) |
| web | tipsnepal.com, bamsawali.com (Nepal's 13-day kriya, described); vedpanchang.com (eclipse *sūtaka*) | 3 | web | as cited |

Not found or not read: no committee (Nepal Panchanga Nirnayak Samiti)
statement on ashauch; no government statement (civil-service *kiriyā* leave
is a separate matter of leave, not of the rite's days). The yearly
committee panchangas read (VS 2082 and 2083) print
half-month tables only, with no ashauch page. The Garuda Purana's
*Sāroddhāra* (the recension commonly read in Nepal) was not read; the
GRETIL text's uttarakhaṇḍa was.

### 1.1 Page map of the DS section (running headers, 1888 edition)

Headers read in the OCR (OCR'd digits corrected so the sequence runs):
298 (line 14475, OCR "९९८"), 299 (14516), 300 (14554, OCR "299"), 301
(14590, OCR "१०१"), 302 (14633, "१०२"), 303 (14671), 304 (14711), 305
(14749, OCR "९०९"), 306 (14786), 307 (14833), 308 (14872, "2०८"), 309
(14908), 310 (14946). Kane cites the same passages of DS from a different
edition (his pp. 427, 433, 435, 436 are our pp. 299, 304, 304–305,
305–306); our numbers are the 1888 edition's.

| DS § | topic | pages |
|---|---|---|
| 35 | *āśauca-nirṇaya* begins; miscarriage, birth, sapinda/samānodaka/sagotra | 298–300 |
| 36 | *mṛtāśauca*: age bands, girls, married daughters, named relations | 300–303 |
| — | *sajyotiḥ* and *pakṣiṇī* defined | 303 |
| 37 | *atikrānta* (heard late), *deśāntara*, the night's thirds, āhitāgni | 303–305 |
| 38 | *āśauca-saṃpāta* (overlap) | 305–306 |
| 39 | *saṃsarga* (contact, carrying, following a bier) | 307–308 |
| 40–43 | *apavāda*: exempt persons, acts, goods; deaths that carry none | 309–313 |

## 2. What each says, clause by clause

### 2.1 What causes it, and what a "day" is

- **The known event, not the event.** Mit on Yā III.18: "jñātam eva jananaṃ
  maraṇaṃ ca nimittam", reasoned from Manu V.75 ("yac cheṣaṃ daśarātrasya")
  which would be pointless if the bare event counted. DS p. 305 defines an
  overlap by knowledge too ("संपातो नाम आशौचिन एकाशौचित्वज्ञाने
  पराशौचित्वज्ञानम्"). Kane IV p. 276 states the rule.
- **A "day" or a "night" means a day-and-night.** Mit on III.18: "atrāśauca-
  prakaraṇe ahargrahaṇaṃ rātrigrahaṇaṃ cāhorātropalakṣaṇārtham". Kane IV
  p. 276.
- **Counting starts from the day of death, or of cremation for an
  āhitāgni.** Mit on III.20, quoting Aṅgiras "anagnim ata utkrānteḥ sāgne
  saṃskārakarmaṇaḥ" and Paiṭhīnasi "anagnim ata utkrānter āśaucaṃ hi
  dvijātiṣu / dāhād agnimato vidyād videśasthe mṛte sati"; NS p. 377
  ("सर्वं चाशौचमाहिताग्नेर्दाहात्, तद्भिन्नस्य मरणमारभ्य ज्ञेयम्", rejecting
  Dhūrtasvāmin's "from death for both"); DS p. 304–305; Dk p. 1114–1115;
  GP 2.34.63 ("yadā tadā bhaved dāhaḥ sūtakaṃ mṛtavāsarāt": whenever the
  cremation, the impurity from the day of death); Kane IV p. 276.
- **An event by day belongs to that day; one at night is assigned by one
  of three rules.** Mit on III.20 (after the menstruation passage, which
  shares the rule): "yadi udayottarakālam utpannaṃ tadā taddivasaprabhṛty
  āśaucāhorātragaṇanā kāryā", and for the night three *kalpas*: (1) before
  *ardharātra*, the previous day ("pūrvadivasaikadeśavyāpitve 'py āśaucasya
  tatpūrvadivasaprabhṛty eva gaṇanā"); (2) "rātriṃ tredhā vibhajyādye
  bhāgadvaye jananādau jāte pūrvadinaṃ grāhyam"; (3) "prāg udayāt", before
  sunrise, the previous day; Kaśyapa's verse holds all three; "eteṣāṃ ca
  kalpānāṃ deśācārato vyavasthā". NS p. 377 attributes the thirds to the
  Mitakshara and leaves the half-night and sunrise to local custom ("तत्र
  देशाचारतो व्यवस्था"). DS p. 304 gives the thirds first ("रात्रौ जननमरणे
  रात्रौ मरणज्ञाने वा रात्रिं त्रिभागां कृत्वा प्रथमभागद्वये पूर्वदिनं तृतीयभागे
  उत्तरदिनमारभ्याशौचम्"), the half-night second ("यद्वार्धरात्रात्प्राक्
  पूर्वदिनं परतः परदिनम्"), "अत्र देशाचारादिना व्यवस्था"; note it applies the
  same rule **to news heard at night**. Dk p. 1114 renders the NS passage
  in Nepali the same way. Pr rule 16: the thirds, "some" sunrise. Kane IV
  pp. 287–288 (§5.2 row K1).
- **The day counts though only part of it is touched** ("ekadeśavyāpitve
  'pi … tatpūrvadivasaprabhṛty eva gaṇanā", Mit on III.20): the event's day
  is day 1.
- **The end of n days is the sunrise that begins day n + 1.** GP 2.5.19
  "daśamaṃ piṇḍam utsṛjya rātriśeṣe śucir bhavet" (pure at the end of the
  night after the tenth pinda); DS p. 305–306's overlap rules treat "the
  tenth night" and "its fourth watch" as inside the first period; Pr rule
  7, "पहिलो मरणाशौचको एघारौं दिनको सूर्योदयपछि" a second death begins a full
  new period, so the first ended at the eleventh day's sunrise; NS on
  naming (Dk p. 1072): "दशम्यामतीतायां विप्रः" names on the 11th, which Dk
  glosses "दशौं दिन पूरा भएपछि एघारौं पुण्य दिनमा".

### 2.2 The short lengths

- **Snāna (*sadyaḥ-śauca*)**: purity by a bath, at once. Kane IV pp. 295–296
  collects the readings (absence of ashauch; bath; part of a day or night).
- **Sajyotiḥ**: DS p. 303, "दिवामरणे रात्रौ स्नानाच्छुद्धिः रात्रिमरणे दिवा
  शुद्धिरिति सज्योतिःपदार्थः": an event by day ends at nightfall, one at night
  at daybreak.
- **Pakṣiṇī**: DS p. 303, by day: "स दिवसः सा रात्रिर्द्वितीयदिवसे
  नक्षत्रदर्शनपर्यन्तम्", that day, that night and the next day until the stars
  show (nightfall of day 2); at night: "सा रात्रिस्तदुत्तरमहो रात्रिश्च", that
  night, the next day and its night (sunrise of day 3); "केचित्" at night
  too, until the stars of the next day. Kane IV p. 278: "two nights with a
  day between or two days with a night between". Pr rule 27: 1.5 days, 36
  hours.
- **Ekāha / naiśikī / ahorātra**: one day-and-night (Mit on III.23, "naiśikī
  niśāyāṃ bhavā ahorātravyāpinī"; Kane IV p. 295 n. 622).
- **Trirātra / tryaha**: three day-and-nights.

### 2.3 Who: the relation

- **Sapiṇḍa to the 7th, 10 days; samānodaka, 3; sagotra, 1 or none.** Manu
  V.59–60 ("daśāhaṃ śāvam āśaucaṃ sapiṇḍeṣu vidhīyate"; "sapiṇḍatā tu puruṣe
  saptame vinivartate / samānodakabhāvas tu janmanāmnor avedane"); V.71
  ("janmany ekodakānāṃ tu trirātrāc chuddhir iṣyate"); Mit on III.18
  ("sapiṇḍānāṃ saptamapuruṣāvadhikānām aviśeṣeṇa daśarātram, samānodakānāṃ
  trirātram", rejecting a graded 4th/5th/6th/7th verse as *vigīta*); DS p.
  299, from the *kūṭastha* to the 7th sapiṇḍas, 7 more samānodakas, to the
  21st sagotras, "सपिण्डानां दशाहम् … सोदकानां त्रिरात्रं सगोत्राणामेकरात्रम्
  इति नागोजीभट्टीये; अन्ये तु सगोत्राणां नाशौचमित्याहुः", the same for birth
  and death; NS p. 369 (Agni Purāṇa: samānodaka to the 14th; Bṛhaspati:
  sapiṇḍas 10, sakulyas 3, gotrajas a bath); GP 2.5.10–11; Pr rules 1–3
  (sapiṇḍa 7 generations 10, sodaka 8th–14th 3, sagotra 15th–21st one
  day-and-night).
- **DS p. 299 on the 7th/8th boundary across branches**: one descended
  through the 8th in one line and another through the 7th: the
  relationship is counted from the nearer side; DS weighs Śūlapāṇi's
  reading via NS and prefers its own second view ("अत्र मम द्वितीयपक्ष एव
  युक्तो भाति"). A request gives the relation already resolved (§3.2), so
  this stays documentation.
- **Named non-sapiṇḍa relations** (DS pp. 301–303; Mit on III.24; Manu
  V.80–82; Pr rules 21–30). The rows the app needs, DS's value first:

  | the deceased, to the observer | DS | Pr |
  |---|---|---|
  | parent, to a married daughter | 3 days inside the 10, a pakṣiṇī after (pp. 301, 303) | 3 (rule 24) |
  | married brother ↔ married sister | 3 if in each other's house, else pakṣiṇī; another village 1 day (p. 301) | 3 (rule 25) |
  | maternal grandfather | 3; another village pakṣiṇī (p. 301) | 3 (rule 23) |
  | maternal grandmother | pakṣiṇī (p. 301) | 3 (rule 23) |
  | maternal uncle | pakṣiṇī; in one's house 3 (p. 301) | 3 (rule 23) |
  | father- or mother-in-law, to the son-in-law | 3 if present, else pakṣiṇī (p. 302) | 3 (rule 23) |
  | son-in-law, to the parents-in-law | 1 day or a bath (p. 302) | 3 (rule 23) |
  | ācārya | 3; another village pakṣiṇī (p. 302; Manu V.80) | 3 (rule 23) |
  | ācārya's wife or son | 1 day (p. 303; Manu V.80) | 1 (rule 30) |
  | adopted son ↔ natural and adoptive parents | 3; their sapiṇḍas 1 (p. 302) | 3 (rule 28) |
  | the three *bandhus* (cousins through a father's or mother's sister or a mother's brother) | pakṣiṇī if upanīta, 1 day if not, 3 in one's house (p. 302) | 1.5 days (rule 27) |
  | the ruler of the land | sajyotiḥ (p. 303; Manu V.82) | — |
  | an ascetic (*yati*) | sapiṇḍas a bath only (p. 303) | — |

  The two agree on most rows and differ in four; the table ships as data
  with each row's clause, so a tradition's row is its own.

### 2.4 How long: the full period

- **By varṇa: 10, 12, 15, 30.** Manu V.83 ("śuddhyed vipro daśāhena
  dvādaśāhena bhūmipaḥ / vaiśyaḥ pañcadaśāhena śūdro māsena śudhyati"); Yā
  III.22 (with 15 for a *nyāyavartin* śūdra); DS p. 299 for birth, p. 301
  for death ("वैश्यादिविशेषः प्रागुक्तोऽत्रानुसन्धेयः"); NS p. 368; GP 2.34.65.
- **Or 10 for all.** Mit on III.22 quotes Aṅgiras "sarveṣām eva varṇānāṃ
  sūtake mṛtake tathā / daśāhāc chuddhir eteṣām iti śātātapo 'bravīt" and
  several other schemes (Parāśara, Śātātapa, Vasiṣṭha), and **declines to
  arrange them**: "teṣāṃ loke samācārābhāvān nātīva vyavasthāpradarśanam
  upayogi". DS p. 299 reports Vijñāneśvara's "सर्ववर्णेषु दशाहं वा" (for
  birth). Pr lists sapiṇḍa 10 with no varṇa at all.
- **What does not vary by varṇa.** Mit on III.21 (Vyāghrapāda "tulyaṃ
  vayasi sarveṣām atikrānte tathaiva ca / upanīte tu viṣamam"): the age
  bands and the late-news rules are the same for all; only the full period
  of an upanīta's death (and of a birth) differs. DS p. 304: "अतिक्रान्ताशौचं
  वयोवस्थानिमित्तकाशौचं च सर्ववर्णसाधारणम्". NS p. 368 the same. This is why
  the API needs **one number**, the full period, and nothing else about
  caste (§3.1).

### 2.5 The deceased's age: the bands

**Mit on III.23** (its own summary, verbatim): "prāṅ nāmakaraṇāt
sadyaḥśaucaṃ, tad ūrdhvaṃ dantajananād arvāg agnisaṃskārakriyāyāṃ ekāhaḥ,
itarathā sadyaḥśaucam, jātadantasya ca prathamavārṣikāc caulād arvāg
ekāhaḥ, prathamavarṣād ūrdhvaṃ trivarṣaparyantaṃ kṛtacūḍasya tryaham,
itarasya tv ekāhaḥ, varṣatrayād ūrdhvam akṛtacūḍasyāpi tryaham,
upanayanād ūrdhvaṃ sarveṣāṃ brāhmādīnāṃ daśarātrādikam". Yā III.23 ("ā
dantajanmanaḥ sadya ā cūḍān naiśikī smṛtā / trirātram ā vratādeśād
daśarātram ataḥ param"); Manu V.67–70; GP 2.5.12. On Yā III.18 the parents
of a child under two keep 10 days ("ūnadvivarṣa ubhayoḥ"), Mit reading it
as the parents' untouchability; DS's bands below give the parents 3 days, and the
`MITAKSHARA` preset keeps Mit's reading.

**DS p. 300** (a son), with its markers: naming = the 12th day, teething =
the 7th month ("नामकरणं द्वादशदिनोपलक्षणम्, दन्तजननं सप्तममासोपलक्षणम्"),
chūḍā or else the end of the 3rd year:

| band (son) | sapiṇḍas | parents | rite |
|---|---|---|---|
| inside the 10 days after birth (after the cord is cut) | the full **birth** period; no death period | the same | — |
| before naming (12th day) | a bath | 3 days | burial only |
| naming to teething (7th month) | 1 day if cremated, a bath if buried | 3 days | either |
| teething to chūḍā (or the 3rd year's end) | 1 day ("केचित्": burial 1, cremation 3) | 3 days | either |
| chūḍā done, or past 3 years, to upanayana | 3 days, all sapiṇḍas; samānodakas a bath | 3 days | cremation |
| after upanayana | the full period; samānodakas 3; sagotras 1 | full | — |

Vijñāneśvara's variant (DS p. 300): from the 11th day to upanayana the
parents keep 3 days throughout.

**A daughter** (DS pp. 300–301): to the 3rd year, sapiṇḍas of three
generations a bath, parents 1 day to the 6th month and 3 from the 7th;
past 3 years to betrothal (*vāgdāna*), sapiṇḍas of three generations 1
day, parents 3; betrothal to marriage, the father's and the groom's
sapiṇḍas 3; after marriage, the husband's sapiṇḍas the full period; dying
at her father's house after marriage, parents, step-mother and brothers 3,
co-resident uncles 1. Mit on III.24 and Manu V.72 agree in substance. Pr
rule 20 matches.

**One whose community has no upanayana** (DS pp. 300–301, of the śūdra):
the 3-day band runs to marriage, or to the 16th year if unmarried
("अस्योपनयनस्थाने विवाहः"). Stated neutrally in the API as: the band's end
is upanayana, or marriage or the 16th year where upanayana is not
performed (§3.2).

**Birth-side bands** (DS pp. 299–300, Mit on III.20, NS pp. 361–364):

| event | mother | father | sapiṇḍas |
|---|---|---|---|
| loss in months 1–3 (*srāva*) | 3 nights (4th month: 4) | a bath | a bath |
| loss in months 5–6 (*pāta*) | as many days as months (5 or 6) | 3 days birth period | 3 days birth period |
| birth from the 7th month, live or still | 10 days untouchable, then barred from rites to the 30th day (son) or 40th (daughter) | bath, then rites allowed | the full birth period |
| child dies before the cord is cut | 10 | 3 | 3 (birth) |
| married daughter delivers at her father's house | — | her parents 1 day (Smṛtyarthasāra; Mādhava: 3) | co-resident brothers 1 |

DS p. 299: "इदं गर्भस्रावपाताशौचं सर्ववर्णसाधारणम्". Mit on III.20 gives
Marīci's graded srāva (3, 4, 5, 8 by varṇa) and reads Vasiṣṭha's 3 days as
the pāta of the 5th–6th months. The birth day itself does not bar gifts
(Mit on III.19, "tad ahar na praduṣyeta"; days 1, 6 and 10 are free for
the *janmadā* worship, Vyāsa there). **No birth period is "heard late"**:
Manu V.77, Mit on III.21 ("janmany atikrāntāśaucaṃ sapiṇḍānāṃ nāsti"),
Devala "nāśuddhiḥ prasavāśauce vyatīteṣu dineṣv api"; the father bathes.

### 2.6 Heard late (*atikrānta*) and abroad (*deśāntara*)

- **Inside the period**: only the days left. Manu V.75; Yā III.21 "proṣite
  kālaśeṣaḥ syāt"; DS p. 304 ("तत्तदाशौचमध्ये ज्ञानेऽवशिष्टदिनैः शुद्धिः");
  Kane IV p. 283.
- **After it, by how long since the death** (DS p. 304, "दशाहोर्ध्वम् एकदेशे
  सपिण्डमरणे ज्ञाते"; Vṛddha-Vasiṣṭha in Mit on III.21 "māsatraye trirātraṃ
  syāt ṣaṇmāse pakṣiṇī tathā / ahas tu navamād arvāg ūrdhvaṃ snānena
  śudhyati"; GP 2.34.66; Kane IV pp. 283–284; Pr rule 12):

  | news reaches | DS (default) | Mādhava (DS p. 304) | Manu V.76 |
  |---|---|---|---|
  | inside the period | the remainder | the remainder | the remainder |
  | after it, within 3 months of the death | 3 days | within 3 fortnights: 3 days | 3 days (to a year) |
  | within 6 months | pakṣiṇī | pakṣiṇī | 3 days |
  | within 9 months | 1 day | within a year: 1 day | 3 days |
  | within a year | sajyotiḥ, or a bath | — | 3 days |
  | after a year | a bath | a bath | a bath |

  DS: "अत्र आपदनापद्विषयत्वेन व्यवस्था" (the two tables answer distress and
  ordinary times), without saying which is which.
- **Only the full period of an upanīta's death is ever "heard late".** Mit
  on III.21; DS p. 303–304: none for the 1- and 3-day bands, for
  samānodakas' 3, for pakṣiṇī relations; a bath only, even if heard inside
  10 days ("त्रिरात्राद्युत्तरं तद्दशाहन्यूनाशौचानां दशाहमध्ये ज्ञानेऽपि
  नातिक्रान्ताशौचं किंतु स्नानमात्रम्").
- **Parents, spouses, co-wives: the full period from the hearing, however
  late.** Mit on III.21 (Paiṭhīnasi "pitarau cen mṛtau syātāṃ dūrastho 'pi
  hi putrakaḥ / śrutvā taddinam ārabhya daśāhaṃ sūtakī bhavet"); NS p.
  377–378; DS p. 304 ("मातापित्रोर्मरणे दूरदेशेऽपि वत्सरान्तरेऽपि श्रवणे
  पुत्रस्य श्रवणप्रभृति दशाहादिपूर्णमेवाशौचम्; दंपत्योः परस्परं … पूर्णदशाहमेव;
  सपत्नीनां परस्परं … दशाहमेव"); step-mother after the 10 days, 3 days;
  an aurasa son's death heard after a year, the parents 3 days. Kane IV p.
  284. Pr rule 13: parents and spouse abroad, heard within 10 days, the 10
  from the death; after, a full 10 from the hearing.
- **Abroad.** DS p. 304: Mādhava's table above for a sapiṇḍa abroad
  (3 fortnights / 6 months / 9 months / a year), against Vijñāneśvara's
  "a bath only after 10 days" (Mit on III.21, "deśāntaramṛtaṃ śrutvā …
  snānena śudhyanti"); "अत्र माधवमतमेव युक्तम्". What counts as abroad: Mit
  (Bṛhaspati) a great river or mountain between, or another language;
  "some" 60, 40 or 30 yojanas; DS p. 304 gives yojana distances graded by
  varṇa and prefers a great mountain or river *with* a change of language.
  Kane IV pp. 284–285 lists the disagreement.

### 2.7 Overlap (*saṃpāta*)

The first rule: a second impurity known during the first ends with the
first, unless it is longer. Manu V.79 ("antardaśāhe syātāṃ cet punar
maraṇajanmanī / tāvat syād aśucir vipro yāvat tat syād anirdaśam"); Yā
III.20a ("antarā janmamaraṇe śeṣāhobhir viśudhyati"); Mit on III.20
(Uśanas: "svalpāśaucasya madhye tu dīrghāśaucaṃ bhaved yadi / na pūrveṇa
viśuddhiḥ syāt svakālenaiva śudhyati"); GP 2.39.12. Then:

1. **A death is not absorbed by a birth.** Mit on III.20 (Aṅgiras "sūtake
   mṛtakaṃ cet syān mṛtake tv atha sūtakaṃ / tatrādhikṛtya mṛtakaṃ śaucaṃ
   kuryān na sūtakam"; Ṣaṭtriṃśanmata "śāvena śudhyate sūtir na sūtiḥ
   śāvaśodhinī"). DS p. 305: a 10-day death absorbs a 10-day or shorter
   death and a 10- or 3-day birth; "जननाशौचेन समेनाधिकेन वा मृताशौचं
   नापैति"; a pakṣiṇī death absorbs neither a 3- nor a 10-day birth, a
   3-day death not a 10-day birth, "इति बहवः"; "कश्चित्" a shorter death
   absorbs a longer birth.
2. **The last night: two days more; its last watch: three.** Gautama in
   Mit on III.20 ("rātriśeṣe sati dvābhyāṃ prabhāte sati tisṛbhiḥ"), read
   there as: when only the last night of the first remains, two more
   nights after it; in the night's last *yāma*, three; Śātātapa
   "rātriśeṣe dvyahāc chuddhir yāmaśeṣe śucis tryahāt". DS pp. 305–306
   ("दशाहान्त्यरात्रौ यदि निवृत्तियोग्यदशाहसंपातस्तदा दिनद्वयमधिकं कार्यम्;
   दशमरात्रेश्चतुर्थयामे … दिनत्रयमधिकम्"), only for a 10-day falling into a
   10-day; a 3-day falling into the 10th night ends with the first; the
   extended days themselves absorb a pakṣiṇī (the two) or a 3-day (the
   three) but not a 10-day. NS p. 381 ("पूर्वाशौचान्त्यरात्रावन्याशौचेऽहोरात्र-
   द्वयमधिकम्, रात्रेरन्त्ययामे तु दिनत्रयम् … एतत् सवर्णाशौचसंपाते एव;
   रात्रिशेषे त्रिरात्रादिसंपाते तु पूर्वशेषेणैव शुद्धिः"). Pr rule 6 has only
   the two days, "दशौं दिनको मध्यरात्रीसम्म".
3. **Mother and father.** Mit on III.20 (a *smṛtyantara*: "mātary agre
   pramītāyām aśuddhau mriyate pitā / pituḥ śeṣeṇa śuddhiḥ syān mātuḥ kuryāt
   tu pakṣiṇīm"), read there: the father dying in the mother's period, the
   father's full period; the mother dying in the father's, the father's
   then a pakṣiṇī more. DS p. 306 the same, the pakṣiṇī only if the mother
   dies before the 10th night (in the 10th night or its last watch, the 2
   or 3 days instead); "स्मृत्यर्थसारादयः" the father's full in the mother's.
   Kane IV p. 294 records Aparārka's other reading. NS pp. 381–382 the
   same as DS.
4. **Not absorbed at all**: the mother of a newborn (*sūtikā*) and the one
   who cremates (DS p. 306, "इयं संपाते पूर्वेण शुद्धिः सूतिकाया अग्निदस्य च
   नास्ति"; Ṣaḍaśīti in Kane IV p. 295); a son's period for a parent by a
   sapiṇḍa's, a wife's for her husband (DS p. 306, "पुत्रस्य सपिण्डाशौचेन
   मातापित्रोराशौचं नापैति; एवं भार्याया भर्त्राशौचं नापैति"). Pr rule 8.
5. **Rites go on.** The preta's pinda during a birth period (Mit on III.20,
   Śātātapa "prārabdhe pretapiṇḍe tu madhye cej jananaṃ bhavet"); the
   father's jātakarma during a death period (Prajāpati, Mit on III.20;
   DS p. 306 "पुत्रजनने जातकर्मादिप्रतिबन्धो नास्ति"; Kane IV p. 295).

Kane IV pp. 292–294 sets out NS's twelve cases (two kinds × three lengths
× which came first); DS p. 305–306 follows NS with the exceptions above.

### 2.8 Exempt persons and occasions (*apavāda*), reported, not computed

DS pp. 309–310, Mit on III.27–29, Manu V.93–98: kings in office, priests
and the consecrated in a sacrifice, a rite once begun (a marriage after
its nāndīśrāddha, a vrata after its sankalpa, Laghu-Viṣṇu in Kane IV p.
291), physicians taking a pulse, national calamity; Pr rules 32–37. These
answer "may this rite go on", a different question; the answer carries no
day. A second request kind can come later (§9).

### 2.9 Nepal's practice

What the texts and the one Nepali summary found say together:

- **Birth: 10 days, naming on the 11th.** NS (Dk p. 1072) quoting Manu
  II.30 ("nāmadheyaṃ daśamyāṃ tu dvādaśyāṃ vāsya kārayet") and Viṣṇu
  ("āśaucavyapagame nāmadheyam"): the naming falls after the period, "on
  the 10th passed, a brāhmaṇa; the 12th passed, a kṣatriya; the 16th, a
  vaiśya; the 31st, a śūdra"; Dhakal's Nepali: the 11th, 13th, 17th and
  22nd days. So **nwaran on the 11th** is a 10-day birth period's first
  pure day, and a family keeping 12 days names on the 13th. Pr rule 17
  starts the birth period at the cutting of the cord (Dk p. 1071 cites
  Jaimini and Hārīta for that, but NS limits it: the birth period runs from
  the birth, the cord rule is for the infant's death period); Pr: the
  mother 30 nights.
- **Death: 10 days of impurity for every sapiṇḍa** (Pr rule 1, no varṇa).
  **The 13 days of kriya** are the rites' span, not the impurity's: the
  ten daily pindas (GP 2.5.18–19, 2.34.40–51), the *ekādaśāha* śrāddha on
  the 11th (GP 2.34.52–55, 59), sapiṇḍīkaraṇa on the 12th (DS p. 331, the
  option `shraddha.md` C673 proposes), the household's purification
  (*śuddhi*) closing it. bamsawali.com and tipsnepal.com (rank 3) describe
  "ten to thirteen days" closing with the śuddhi; no rank-1 or rank-2
  statement that the impurity itself is 13 days was found. The design
  keeps the two apart: ashauch answers the impurity; the first-year
  schedule (`shraddha.md` §2.6, C673) answers the 11th, 12th and 13th
  days. A family that keeps 13 days of jutho sets `full_days: 13`.
- **The committee** publishes no ashauch rules that this search found.

### 2.10 What other apps compute

No published description of a birth or death ashauch calculator was found
for Drik Panchang or any other almanac app; Drik's and vedpanchang's
"sutak" pages are the eclipse *sūtaka* (vedpanchang.com's eclipse tables:
12 hours before a solar eclipse, 9 before a lunar). Drik's own pages were
not fetched (reCAPTCHA). The vedadhara.com article found states that a
second death "extends" the impurity, which no text read here says (the
texts' rule is the reverse, §2.7). Recorded as a gap, not a source.

## 3. The forks, each with a proposed default

| # | fork | proposed default | evidence | other readings, all offered |
|---|---|---|---|---|
| F1 | the tradition table | **`DHARMASINDHU`**: DS pp. 298–306, the same edition the festival rules read, and the most detailed; Nepal's own commentary tradition (Dhakal on NS) agrees with it clause by clause where read | DS; NS; Dk | `MITAKSHARA` (Mit's own table on III.23, Vijñāneśvara's abroad bath, its night kalpas); `NEPAL` = DS with Pr's four relation rows, the two-day-only last-night rule, and the thirds |
| F2 | the full period | **a number, default 10** (`full_days`); the texts' varṇa lengths are documentation, never an input | Manu V.83, Yā III.22 vs Aṅgiras in Mit on III.22; Mit declines to arrange the schemes; Pr has 10 for every sapiṇḍa | 12, 15, 30, 13 or any number a family keeps |
| F3 | the night | **thirds**: the night (sunset to sunrise) in three; the first two belong to the day before | Mit's second kalpa, which NS p. 377 calls the Mitakshara's; DS p. 304 gives it first; Pr rule 16 | `HALF_NIGHT` (before the night's middle, the day before); `SUNRISE` (any time before sunrise, the day before); see F3a |
| F3a | "ardharātra" | **the middle of the night**, sunset to sunrise, as `Window::Nishitha` | ardha-rātra is half the night; the SDK's niśītha already is that instant | civil midnight (Kane IV p. 287 writes "midnight"; Pr "मध्यरात्री"); offered as `CIVIL_MIDNIGHT` |
| F4 | the first day | **the event's day is day 1**, however little of it is left | Mit on III.20 "ekadeśavyāpitve 'pi … tatpūrvadivasaprabhṛty eva gaṇanā"; the 11th-day naming | Kane IV pp. 287–288's illustration reads as if the event's day were not counted (§5.2 row K1); not offered until a second source says so |
| F5 | the end | **the sunrise that begins day n + 1** | GP 2.5.19; DS pp. 305–306; Pr rule 7 | — |
| F6 | counting from | **the death**; from the cremation when `ahitagni` is set | Mit (Aṅgiras, Paiṭhīnasi), NS p. 377, DS p. 305, GP 2.34.63 | Dhūrtasvāmin's "from death for all" (NS p. 377, refused) |
| F7 | pakṣiṇī at night | **that night, the next day and its night** (to the second sunrise) | DS p. 303 | "केचित्": to the next day's nightfall (DS p. 303) |
| F8 | late news after the period | **DS's table** (3 months / 6 / 9 / a year) | DS p. 304, Vṛddha-Vasiṣṭha in Mit, GP 2.34.66, Pr rule 12 | Mādhava's (3 fortnights / 6 months / a year); Manu V.76's (3 days to a year) |
| F9 | late news, abroad | **Mādhava's table** | DS p. 304 "अत्र माधवमतमेव युक्तम्" | Vijñāneśvara's bath only (Mit on III.21) |
| F10 | whether "abroad" | **a request fact**, `abroad: bool`, never derived | the texts disagree on the test and DS's distances are graded by varṇa | — |
| F11 | the last night of a 10-day period | **+2 days; in its last watch (the night's last quarter), +3** | Gautama and Śātātapa in Mit on III.20; DS pp. 305–306; NS p. 381 | Pr rule 6, +2 only before the 10th day's midnight |
| F12 | a shorter death and a longer birth | **neither absorbs the other** ("बहवः") | DS p. 305 | "कश्चित्": the death absorbs the birth (DS p. 305) |
| F13 | mother then father | **Mit's reading**: father's full period in the mother's; mother's in the father's, the father's then a pakṣiṇī (2 or 3 days instead when in the 10th night) | Mit on III.20, DS p. 306, NS pp. 381–382 | the Smṛtyarthasāra's (father's full in the mother's, no pakṣiṇī rule) |
| F14 | the mother's bar on rites after the birth period | **reported as a second span**, to the 30th day after a son and the 40th after a daughter | DS p. 299 (Paiṭhīnasi in Mit on III.19, 20 and 30 after the 10) | Pr rule 17, 30 nights without the distinction; off |
| F15 | the teething-to-chūḍā band | **1 day for sapiṇḍas** | DS p. 300, Mit on III.23 | "केचित्": burial 1, cremation 3 (DS p. 300) |
| F16 | parents of a child before upanayana | **DS's bands** | DS p. 300 | Vijñāneśvara's 3 days from the 11th day to upanayana (DS p. 300) |
| F17 | sagotra | **1 day** | DS p. 299 (Nāgojī-bhaṭṭīya); Pr rule 3 | none ("अन्ये", DS p. 299); Bṛhaspati a bath (NS p. 369) |

## 4. The mapping onto the code

### 4.1 What exists

- The day as sunrise to sunrise, with its sunset and the next sunrise
  (`Panchanga.day.sunrise/next_sunrise`, `window`), and the instant's day
  (`Almanac::holding`, `almanac.rs` line ~725), as the sankalpa uses.
- The night's middle (`Window::Nishitha`) and the night's muhurtas
  (`Window::NightMuhurta`), so the night's thirds and last quarter are a
  fraction of sunset → next sunrise the evaluator already measures.
- `Origin::Instant { at }` and the first-year schedule in
  `festival::anniversary` (`shraddha.md`), which the 11th/12th-day rites
  hang from.
- The envelope, provenance and catalogue-key conventions every answer
  carries; `INVALID_ARG` / `OUT_OF_RANGE` / `CAPABILITY` refusals as the
  sankalpa's (§4 of `sankalpa.md`).

Nothing about the tithi is involved: ashauch is a count of sunrise days,
not a lunar rule, so **no `FestivalRule` addition is needed**. The new
code is a small module, `teistro_panchanga::ashauch`, with a data table.

### 4.2 What was built

`teistro_panchanga::ashauch` (re-exported as `teistro::ashauch`), its
kernel `ashauch(table, request, days, clock)` over a run of sunrise days,
and `Almanac::ashauch`, which finds the days around each instant the
request names (`AshauchRequest::anchors`: the count's start, the
hearing, a recent birth, each running period's end), the day before
through `AshauchRequest::reach` days after, joining runs that meet. News
heard years after the event costs two short runs, not the years between.

```text
AshauchRequest { key, event, relation, heardAt?, abroad, nearness, fullDays?, during: [Running] }
AshauchEvent   BIRTH { at, child? } | LOSS { at, month } | DEATH { at, crematedAt?, ahitagni, deceased }
Deceased       { born?, sex?, chudakarana?, upanayana?, betrothed?, married?, buried }
Relation       MOTHER FATHER CHILD SPOUSE CO_WIFE STEP_CHILD SAPINDA SAMANODAKA SAGOTRA
               MARRIED_DAUGHTER MARRIED_SIBLING MATERNAL_GRANDFATHER MATERNAL_GRANDMOTHER
               MATERNAL_UNCLE SON_IN_LAW PARENT_IN_LAW PUPIL ACHARYAS_FAMILY ADOPTION BANDHU
               SUBJECT OF_ASCETIC
Nearness       OBSERVERS_HOUSE | SAME_VILLAGE (default) | OTHER_VILLAGE
Running        { key, kind, relation, sex?, start, end, length, days? }   (Running::from(&answer))
AshauchTable   { name, night, pakshiniAtNight, relations, bands, losses, late, lateAbroad,
                 overlap { lastNight: [2, 3], deathAbsorbsLongerBirth }, motherRites? }
Ashauch        { key, kind, relation, sex?, start, dayOne, dayOneBy, end, lastDay?, length,
                 days?, clause, overlap?, ritesFrom?, table }
Length         BATH | SAJYOTIH | PAKSHINI | DAYS { days } | FULL
Clause         RELATION | BAND | LOSS { row, said } | BIRTH_PERIOD_RUNS | REMAINDER
               | LATE { within_days, abroad } | LATE_BATH | ROW_HEARD_LATE { row, said }
               | SHORT_HEARD_LATE | BIRTH_HEARD_LATE
Overlapped     ENDS_WITH_EARLIER | LAST_NIGHT { days } | MOTHER_AFTER_FATHER, each { earlier }
```

`sdk.almanac().ashauch(table, request, place, offset)` in the façade
(`ashauch_query` for the binding record `{latitudeDeg, longitudeDeg,
altitudeM, utcOffsetSeconds, table?, request}`), `ts_almanac_ashauch` at
the C boundary, `almanac.ashauch` as an agent tool, and every binding.
The input hash covers the table, the request and the place, so a
family's own table is as reproducible as the preset.

Where the build differs from the draft above, and why:

- **A fact is optional, and asked for rather than guessed.** With no
  `born`, the deceased is an adult who had every rite. With a `born`, a
  band that turns on a fact the request leaves out (the upanayana before
  the 16th year, a daughter's betrothal or marriage, the sex) is refused
  as `INVALID_ARG` named by that field. Left out, the cūḍākaraṇa is read
  from the age (DS p. 300's "or the third year's end"). Ages are counted
  in days of 24 hours from the birth, months of 30 days and years of
  365.2425.
- **Absorption is by length, not by end.** A 10-day death known on day 4
  of another 10-day death ends with the first (T18) although its own ten
  would end later. So a new period is taken in when it is no longer than
  the running one, weighed in half days (a bath 0, a sajyotiḥ 1, a
  pakṣiṇī 3, a count twice its days). It is never extended past its own
  end.
- **The last-night rule needs equal counts of one kind** (NS p. 381,
  "सवर्णाशौचसंपाते एव"): a 3-day death in the 10th night ends with the
  first (T21).
- **`during` takes `Running` records**, the parts of an earlier answer
  the overlap rules read, rather than whole answers, so a request carries
  no calendar date and every binding writes it from its own answer type.
- **The late tables count days**, since Mādhava's first row is three
  fortnights (45 days): `DHARMASINDHU`'s are 90 / 180 / 270 / 360 at home
  (3 days, pakṣiṇī, 1 day, sajyotiḥ) and 45 / 180 / 360 abroad (3 days,
  pakṣiṇī, 1 day), then a bath.
- **A loss is its own rows** (months 1–3, 4, 5, 6), the mother's length
  and the father's and the sapiṇḍas'; any other relation a bath.
- **A relation's own late length** (`heardLate`) carries Paiṭhīnasi's
  rule (a child, a spouse, a co-wife: the full period from the hearing),
  the step-mother's three days and the married daughter's pakṣiṇī, so the
  late tables answer only a sapiṇḍa's full period.
- **The night's civil midnight** is read from the place's clock at that
  night's sunset, so a daylight-saving change moves it with the clock.
- **One preset ships, `DHARMASINDHU`.** `NEPAL` would take Pr's rows,
  which are rank 3 (`CLEAN_ROOM.md`: a rule with no rank-1 citation does
  not ship); a family that keeps them loads them as a table. `MITAKSHARA`
  waits on its own band table (Mit on III.18's ten days for a child under
  two), recorded in §9.
- **The draft's `Relation::Row`** is not needed: a consumer's own row
  names an existing relation under a placement or a married-daughter
  condition, and a relation the table names no row for is
  `UNSUPPORTED`, with a hint, rather than a guess. The parents of a
  married daughter who dies away from their house are such a case
  (C726).

### 4.3 The table is data

The presets are Rust constructors, and every row serialises, so a
consumer can load a table from JSON (as the override packs do) and change
any row: a community that keeps 12 days for a maternal uncle, or 13 for
sapiṇḍas, edits a row or `full_days`, never code (`no-dead-ends`). A row
carries its own clause and source; a row a consumer adds carries the
consumer's source string, and the answer repeats it.

## 5. Hand-checkable cases

### 5.1 From the texts (undated, exact)

| # | case | expected | source |
|---|---|---|---|
| T1 | sapiṇḍa's death, observer an upanīta sapiṇḍa, full_days 10 | `Days{10}` from the death's day | Manu V.59; DS p. 301 |
| T2 | the same, samānodaka | 3 days | Manu V.64 ("tryahād udakadāyinaḥ"), Mit on III.18; DS p. 299 |
| T3 | the same, sagotra | 1 day (none under "अन्ये") | DS p. 299 |
| T4 | son dies on day 30 of life, cremated, observer a sapiṇḍa / the father | 1 day / 3 days | DS p. 300 |
| T5 | son dies before the 12th day, after the cord was cut (day 8), observer a sapiṇḍa | the birth period runs to its own end; no death period | DS p. 300; Mit on III.20 (Bṛhanmanu) |
| T6 | son dies at 4, no upanayana, observer a sapiṇḍa | 3 days | DS p. 300; Mit on III.23 |
| T7 | daughter dies at 2, observer her father / a sapiṇḍa | 3 days (7th month on) / a bath | DS p. 300 |
| T8 | daughter dies at 5, unbetrothed, observer a sapiṇḍa of 3 generations / her father | 1 day / 3 days (a sapiṇḍa beyond three generations: none stated) | DS p. 301 |
| T9 | loss in the 2nd month: mother / father | 3 nights / a bath | DS p. 299 |
| T10 | loss in the 5th month: mother / sapiṇḍas | 5 days / 3 days (birth) | DS p. 299 |
| T11 | sapiṇḍa's death heard on day 6 | the remaining days to day 10's end | Manu V.75; DS p. 304 |
| T12 | the same heard 50 days after the death | 3 days from the hearing | DS p. 304; Mit (Vṛddha-Vasiṣṭha) |
| T13 | heard 5 months after | pakṣiṇī | as T12 |
| T14 | heard 8 months after | 1 day | as T12 |
| T15 | heard 14 months after | a bath | as T12; Manu V.76 |
| T16 | father's death heard 14 months after, observer his son | 10 days from the hearing | Mit (Paiṭhīnasi); DS p. 304 |
| T17 | a birth heard after its 10 days, observer the father / a sapiṇḍa | a bath / nothing | Manu V.77; Mit on III.21 |
| T18 | a 10-day death known on day 4 of another 10-day death | ends with the first | Manu V.79; DS p. 305 |
| T19 | a 10-day death known in the 10th night | the first's end + 2 days | Gautama in Mit; DS p. 306 |
| T20 | known in the 10th night's last quarter | + 3 days | as T19 |
| T21 | a 3-day death known in the 10th night of a 10-day | ends with the first | DS p. 306; NS p. 381 |
| T22 | a death known during a 10-day birth period | the death's full period from its own day | Mit (Aṅgiras); DS p. 305 |
| T23 | a 10-day birth during a 10-day death | ends with the death | DS p. 305 |
| T24 | father dies in the mother's period, observer their son | the father's full period | Mit on III.20; DS p. 306 |
| T25 | mother dies on day 4 of the father's period | the father's end + pakṣiṇī | as T24 |
| T26 | mother dies in the father's 10th night | the father's end + 2 days (no pakṣiṇī) | DS p. 306 |
| T27 | the birthing mother during a sapiṇḍa's death period | her own 10 days, not absorbed | DS p. 306 |
| T28 | married daughter, her father's death on day 3 / on day 40 | 3 days / pakṣiṇī | DS pp. 301, 303 |
| T29 | naming day for a birth on day 1, full_days 10 / 12 | day 11 / day 13 | NS via Dk p. 1072 |

### 5.2 Dated at Kathmandu (sun computed here, to be held to the SDK's)

Sunrise and sunset at Kathmandu (27.7172° N, 85.3240° E, 1337 m), NPT,
refracted upper limb, pyswisseph Moshier, by
a short script: January 2026 sunrise 06:56 (06:55 from the
19th), sunset 17:26 (10th) to 17:38 (24th). Night of Sun 11 → Mon 12
January: sunset 17:27, thirds end 21:57 and 02:26, niśītha 00:12, last
quarter from 03:34, sunrise 06:56. The SDK's own sunrise convention may
differ by minutes; every instant below is at least 30 minutes from a
boundary.

| # | case | thirds (default) | half-night | sunrise rule |
|---|---|---|---|---|
| K1 | **Kane's illustration** (IV pp. 287–288: a death at 1 a.m. on a Monday) made concrete: a sapiṇḍa dies 01:00 Mon 12 Jan 2026 | day 1 Sun 11; ends Wed 21 Jan 06:55 | day 1 Mon 12; ends Thu 22 Jan 06:55 | day 1 Sun 11; ends Wed 21 Jan |
| K2 | a death at 03:30 Mon 12 Jan | day 1 Mon 12; ends Thu 22 Jan | day 1 Mon 12 | day 1 Sun 11 |
| K3 | a death at 22:30 Sun 11 Jan | day 1 Sun 11; ends Wed 21 Jan | day 1 Sun 11 | day 1 Sun 11 |
| K4 | a birth at 01:00 Mon 12 Jan: naming day (full 10) | Wed 21 Jan | Thu 22 Jan | Wed 21 Jan |
| K5 | first death 10:00 Sun 11 Jan (ends Wed 21 06:55); a second sapiṇḍa death 14:00 Thu 15 | ends Wed 21 Jan 06:55, `EndsWithEarlier` | same | same |
| K6 | the same, second at 23:00 Tue 20 Jan (the 10th night) | ends Fri 23 Jan 06:54, `ExtendedLastNight{2}` | same | same |
| K7 | the same, second at 04:15 Wed 21 Jan (the 10th night's last quarter, from 03:35) | ends Sat 24 Jan 06:54, `ExtendedLastNight{3}` | same | same |
| K8 | the same, second at 08:00 Wed 21 Jan (after the first ended) | its own: day 1 Wed 21; ends Sat 31 Jan at sunrise | same | same |
| K9 | a sapiṇḍa dies 10:00 Thu 1 Jan 2026; the news at 12:00 Mon 5 Jan | `Remainder`: ends Sun 11 Jan at sunrise | same | same |
| K10 | the same, news 12:00 Fri 20 Feb (within 3 months) | 3 days from Fri 20 Feb; ends Mon 23 Feb at sunrise | same | same |

K1 is the one place a rank-2 worked example exists. Kane writes that on
the midnight view "Monday would be excluded" and on the thirds "Monday
would be included" in the ten days. Under F4 the thirds give day 1 =
Sunday, so Monday is day 2 and included, which matches. But the midnight
view gives day 1 = Monday, also included. Kane's "excluded" only follows if
the event's day is not counted at all, against the Mitakshara's
"tatpūrvadivasaprabhṛty eva gaṇanā". Check the page image (leaf 324–325 of
the archive scan) before relying on either sentence (crux C716).

### 5.3 Still needing a printed or a priest's answer

- A Nepali family's printed death notice or a committee statement giving
  the jutho's last day for a dated death (none found).
- Whether Chhetri families in Nepal keep 13 days of impurity or 13 days of
  rites (§2.9; F2 covers either by `full_days`).
- A dated night birth with its naming day, to test F3 against practice.

## 6. Crux rows (register format; the integrator numbers them)

| # | item | what is known | what is missing | rank of the evidence | action |
|---|---|---|---|---|---|
| C713 | ashauch: the tradition default | DS pp. 298–306 is the most detailed table; Mit on Yā III.23 has its own band summary; Pr (rank 3) is the one Nepali rule list found and agrees with DS on most rows | the maintainer's choice of default (brief §4) | 1 | built: `DHARMASINDHU`, every row data; `NEPAL` not shipped (its rows are rank 3), `MITAKSHARA` later (§9) |
| C714 | the full period by varṇa | Manu V.83 and Yā III.22 grade 10/12/15/30; Aṅgiras (in Mit on III.22) gives 10 for all; Mit declines to arrange the schemes "for want of usage"; age bands and late news are the same for all (Mit on III.21, DS p. 304, NS p. 368) | none needed: the API takes the number | 1 | `full_days`, default 10; no caste field anywhere |
| C715 | an event at night | three kalpas in Mit on III.20, "by local usage"; NS p. 377 names the thirds the Mitakshara's; DS p. 304 gives them first; Pr rule 16 the thirds | a Nepal statement or a dated case | 1 | default `THIRDS`; `HALF_NIGHT` (the night's middle), `CIVIL_MIDNIGHT`, `SUNRISE` |
| C716 | Kane's 1 a.m. illustration | Kane IV pp. 287–288 says "Monday excluded" under the midnight view, which follows only if the event's day is not day 1; Mit counts it as day 1 | the page image; a second rank-2 statement | 2 | open; F4 default from Mit |
| C717 | where n days end | the sunrise of day n + 1 (GP 2.5.19, DS pp. 305–306, Pr rule 7) | — | 1 | closed by the texts |
| C718 | late news after the period | DS's 3/6/9-month table, Mādhava's 3-fortnight table ("by distress or not", DS p. 304), Manu V.76's 3 days to a year | which table answers distress | 1 | default DS's; others offered |
| C719 | late news from abroad | Mādhava's table vs Vijñāneśvara's bath; DS p. 304 prefers Mādhava; the test for "abroad" differs by source and by varṇa | — | 1 | default Mādhava's; `abroad` a request fact |
| C720 | the last night's extension | +2 in the last night, +3 in its last watch (Gautama, Śātātapa in Mit; DS; NS p. 381), only between 10-day periods; Pr rule 6 +2 before midnight only | whether Nepal keeps the +3 | 1 | built: `lastNight: [2, 3]`; a table may set `[2, 2]` |
| C721 | the mother's bar on rites after a birth | DS p. 299: to the 30th day after a son, the 40th after a daughter; Pr rule 17: 30 nights | whether an app shows it (it differs by the child's sex) | 1 | reported as `rites_from` when the observer is the mother, switchable off |
| C722 | named relations | DS pp. 301–303 and Pr rules 21–30 differ on four rows (§2.3) | a Nepal statement for those rows | 1 vs 3 | each row data with its clause; Pr's rows a family's own table |
| C723 | the 13-day kriya in Nepal | rank-3 descriptions only; the texts' impurity is 10 (12 for a kṣatriya); the 11th–13th days carry rites | a committee or priest statement | 3 | kept apart: ashauch answers the impurity, the first-year schedule (C673) the rites |
| C724 | deaths that carry no ashauch | DS pp. 310–313, Yā III.6, III.21, Manu V.89–90: suicide, some violent deaths, the "fallen"; Pr rule 33 suicide | — | 1 | **not shipped** (§8): the request has no cause-of-death field |
| C725 | a married daughter told inside the ten days | DS p. 301 "दशाहमध्ये त्रिरात्रम्", p. 303 a pakṣiṇī after; the clause does not say whether her three nights count from the death or the hearing | a second rank-1 statement | 1 | built: her three days count from the death, news inside them keeps the days left, news past them takes the row's pakṣiṇī from the hearing |
| C726 | a married daughter's death away from her parents' house | DS p. 301 gives her parents three days when she dies in her father's house; no length for elsewhere was read | the clause for elsewhere | 1 | `UNSUPPORTED` with a hint: a family adds its row |
| C727 | a child's death, the relations its band does not name | DS pp. 300–301's bands name the parents, the sapiṇḍas and the samānodakas | a clause for the maternal kin of a child | 1 | built: a bath, the band's clause cited |

## 7. The order of work, each step with its test

1. **The day of an instant under a night rule** (`NightRule`, built on the
   almanac's day and the night's middle). Test: K1–K3 under all four rules
   at Kathmandu; a polar-day place refused as the sankalpa refuses.
2. **Lengths to instants** (`Bath`, `Sajyotih`, `Pakshini` both ways,
   `Days{n}`, `Full`). Test: each length from a day event and a night
   event, ending on the sunrise or nightfall §2.2 names; K5's end.
3. **The DS table as data**: bands (§2.5), relations (§2.3), late (§2.6),
   with `dharmasindhu()`, `mitakshara()`, `nepal()`; serde both ways; a
   schema. Test: T1–T17 and T28–T29 as unit cases; a JSON round trip of
   each preset; a consumer row overriding a preset row changes only that
   answer.
4. **Late news and abroad.** Test: K9, K10, T11–T17.
5. **Overlap** over `during`. Test: T18–T27 and K5–K8, each proved red by
   breaking its rule (`break-it-to-find-the-gate`).
6. **The façade, the C boundary, the agent tool and every binding**, the
   answer's members held to serde (`hold-a-spelling-to-serde`), the
   clause keys named in `en-Latn` and `ne-Deva-NP` (Nepali from the texts'
   own terms: जुठो, सूतक, पक्षिणी, सज्योति, अतिक्रान्ताशौच). Test: the
   bindings' parity suite on K1–K10.
Steps 1 to 6 are built, each test proved red by breaking its rule; the
bindings' suites hold the Kathmandu case, its late news and an overlap.
The clause keys' locale names wait on a Nepali reader (§8).

7. **The `shraddha.md` first-year schedule** counts its 11th day from the
   ashauch's day one under the same night rule, so the two never disagree
   on which day is the 11th. Built: K1's ten days end on the schedule's
   11th day under each night rule (`crates/sdk/tests/first_year.rs`).

## 8. Sensitivities

- **Caste.** The texts key the full period, the "abroad" distance and some
  srāva lengths on varṇa. The API takes **only a number** (`full_days`)
  and the deceased's rites; it never asks or stores a caste. A preset's
  documentation may cite Manu V.83, but no answer names a varṇa. The one
  place the texts substitute marriage for upanayana (DS p. 301) is
  expressed as `Rite::NotPerformed`, a fact about the deceased's
  community's rites, not a label.
- **Not shipped**: the contact and carrying rules graded by the caste of
  the dead and the carrier (DS pp. 307–308), the pratiloma and slave rules
  (Mit on III.22, NS p. 376), the deaths that "carry no ashauch" (suicide,
  certain violent deaths, the "fallen"; DS pp. 310–313, C724), and the
  menstruation rules that share Mit's passage. An almanac telling a
  grieving family that a death by suicide carries no mourning would do
  harm the texts' own later commentators already soften (DS p. 311
  prescribes the rites after expiation); none of it is needed to answer
  "until when".
- **Sex.** The girl bands, the married daughter's rows and the mother's
  rites bar (F14) differ by sex in the texts. They ship as the texts give
  them, each row data a family can change, and the mother's bar is
  reported only to the mother and can be switched off.
- **Grief.** The answer is a date and a clause. Message wording in the
  locale packs should be plain and neutral ("impurity ends at sunrise on
  …"), reviewed by a Nepali reader, never a machine stub.

## 9. Not yet

- *Apavāda* as its own question ("may this rite go on during the
  period?", §2.8).
- A household view: several events and several observers at once, which
  is the same request repeated with `during` filled; a convenience
  wrapper can come after step 6.
- The kriya days themselves (11th, 12th, 13th) belong to `shraddha.md`'s
  first-year schedule (C673, §3.3), built: its 11th day is this page's day
  one plus ten under the same night rule.
