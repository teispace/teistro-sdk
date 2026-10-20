# Festival classification

Status: `research`, 2026-10-20. Cruxes C600, C613, C619 and C622 to C623
(`01-research/feature-universe/19-verification-cruxes.md`). The rules this
page counts as shipped are `festival-rules.md` §9 and §10. The forms it
counts as needed are `festival-rule-language.md`.

A consumer that keeps Nepal's festivals asked which of them the SDK can
reckon, and how. This page answers for every festival in that consumer's
list. Each is named by an SDK key and its names, and sorted by what decides
its day:

| class | what decides the day |
|---|---|
| **rule, shipped** | a festival rule in a pack the SDK ships (`NEPAL` or `NEPAL_PANCHANGA`) |
| **rule, not yet shipped** | a tithi in a month, at sunrise or in a window, which the language holds today; the consumer's source reproduces each at sunrise, and no pack carries it yet |
| **rule, needs a form** | a rule the language cannot say yet, or a rule the evidence does not settle (`festival-rule-language.md` §4) |
| **solar** | a day of the solar month, or a sankranti (C619) |
| **fixed civil** | a Bikram Sambat or Gregorian date fixed by law, an institution or an organisation |
| **community** | a day one community sets each year, with no rule in hand |
| **published only** | a day an authority announces: a moon sighting, a government holiday, a one-off event |

The community column names who keeps a festival when it is one
community's. A blank means the festival is kept across communities, or
that the community is not known here.

## 1. What "663" counts

The consumer's name map holds 663 lines: a header and 662 event names, as
its source spells them. Of the 662, 565 map to a festival, and several
names may map to one, so the 565 name 502 festivals. The other 97 map to
none. The festival list holds the same 502. So "about 663 IDs" counts
names and not festivals: **502 festivals, and 97 names that are not one**.
§5 classifies the 97.

## 2. Against track 04's counts

Track 04 counted the 502 by the kind of rule the source gives: 295 lunar
(57 not reproducible), 68 Bikram Sambat, 113 Gregorian, 2 both and 24 with
no rule. This page's classes split each of those counts:

| track 04 | count | this page |
|---|---|---|
| lunar | 295 | rule, shipped 82; rule, not yet shipped 173; rule, needs a form 40 |
| Bikram Sambat | 68 | solar 24; fixed civil 44 |
| Gregorian | 113 | fixed civil 113 |
| both | 2 | solar 1 (Guriya parva, printed on Shrawan 1 in both editions, C621); rule, needs a form 1 (Walke parva) |
| no rule | 24 | rule, shipped 2; fixed civil 8; solar 1; community 6; published only 5; rule, needs a form 2 |

Where the two counts differ, and why:

- **"57 not reproducible" is not a lunar count.** It counts the reviewed
  table's 57 excluded rows. Only 39 of them have a lunar rule in the source.
  Of the rest, 14 have no rule, 2 are Bikram Sambat dates, 1 is Gregorian
  and 1 is both.
- **The table's excluded rows are not all excluded here.** Janmashtami is
  among them, and the SDK ships the text's nirnaya rule for it (C616). It is
  counted as shipped.
- **The "no rule" festivals are not all unreckonable.** The reviewed table
  gives a computable rule to ten of the 24. Two of those are now shipped:
  the Nepal Sambat new year, which is kept on Mha puja's day, and the start
  of Vaishakha's bathing, on Chaitra's full moon. Five are Gregorian weekday or date
  rules, and three are Bikram Sambat dates.

Total: 502 festivals: rule, shipped 84; rule, not yet shipped 173; rule, needs a form 43; solar 26; fixed civil 165; community 6; published only 5.

## 3. The festivals, by class


### Rule, shipped (84)

The key is the shipped rule's. `NEPAL_PANCHANGA` holds `NEPAL`, so every key below is in it.

| key | name | नाम | community | pack |
|---|---|---|---|---|
| `AJA_EKADASHI` | Aja Ekadashi | अजा एकादशी |  | NEPAL_PANCHANGA |
| `AMAVASYA_SHRADDHA` | Aunsi Shraddha | औँसी श्राद्ध |  | NEPAL_PANCHANGA |
| `AMUKTABHARANA_SAPTAMI` | Amuktabharana Saptami | अमुक्ताभरण सप्तमी |  | NEPAL_PANCHANGA |
| `ANALA_PUNHI` | Anala Punhi | अनला पुन्ही |  | NEPAL_PANCHANGA |
| `ASHTAMI_SHRADDHA` | Ashtami Shraddha | अष्टमी श्राद्ध |  | NEPAL_PANCHANGA |
| `BALA_CHATURDASHI` | Bala Charhe | बाला चह्रे | Newar | NEPAL_PANCHANGA |
| `BALI_PRATIPADA` | Bali Puja | बलि पूजा |  | NEPAL |
| `BHIMA_EKADASHI` | Bhima Ekadashi | भीमा एकादशी |  | NEPAL_PANCHANGA |
| `BHISHMA_PANCHAKA_ARAMBHA` | Bhishma Panchak Begins | भीष्मपञ्चक आरम्भ |  | NEPAL_PANCHANGA |
| `CHAITE_DASHAIN` | Chaite Dashain | चैते दसैँ |  | NEPAL_PANCHANGA |
| `CHAMPA_SHASHTHI` | Champa Shashthi | चम्पा षष्ठी |  | NEPAL_PANCHANGA |
| `CHATHA` | Chatha | चथा | Newar | NEPAL_PANCHANGA |
| `CHATURDASHI_SHRADDHA` | Chaturdashi Shraddha | चतुर्दशी श्राद्ध |  | NEPAL_PANCHANGA |
| `CHATURTHI_SHRADDHA` | Chaturthi Shraddha | चतुर्थी श्राद्ध |  | NEPAL_PANCHANGA |
| `DASHAHARA_SNANA_ARAMBHA` | Dashahara Snan Begins | दशहरा स्नान आरम्भ |  | NEPAL_PANCHANGA |
| `DASHAMI_SHRADDHA` | Dashami Shraddha | दशमी श्राद्ध |  | NEPAL_PANCHANGA |
| `DATTATREYA_JAYANTI` | Dattatreya Jayanti | दत्तात्रेय जयन्ती |  | NEPAL_PANCHANGA |
| `DHANVANTARI_JAYANTI` | Dhanvantari Jayanti | धन्वन्तरि जयन्ती |  | NEPAL_PANCHANGA |
| `DILA_CHARHE` | Dila Charhe | दिला चह्रे | Newar | NEPAL_PANCHANGA |
| `DURVA_ASHTAMI` | Durva Ashtami | दूर्वाष्टमी |  | NEPAL_PANCHANGA |
| `DVAPARA_YUGADI` | Dwaparyugadi | द्वापरयुगादि |  | NEPAL_PANCHANGA |
| `DWADASHI_SHRADDHA` | Dwadashi Shraddha | द्वादशी श्राद्ध |  | NEPAL_PANCHANGA |
| `DWITIYA_SHRADDHA` | Dwitiya Shraddha | द्वितीया श्राद्ध |  | NEPAL_PANCHANGA |
| `EKADASHI_SHRADDHA` | Ekadashi Shraddha | एकादशी श्राद्ध |  | NEPAL_PANCHANGA |
| `GAIDU_PUJA` | Gaidu Puja | गैडु पूजा |  | NEPAL_PANCHANGA |
| `GANESHA_CHATURTHI` | Ganesh Chaturthi | गणेश चतुर्थी |  | NEPAL_PANCHANGA |
| `GANGA_DASHAHARA` | Ganga Dashahara | गङ्गा दशहरा |  | NEPAL_PANCHANGA |
| `GANGA_SAPTAMI` | Ganga Saptami | गङ्गा सप्तमी |  | NEPAL_PANCHANGA |
| `GAURA_SAPTAMI` | Gaura Saptami | गौरा सप्तमी |  | NEPAL_PANCHANGA |
| `GHODE_JATRA` | Ghode Jatra | घोडेजात्रा | Newar | NEPAL_PANCHANGA |
| `GOSAIKUNDA_SNANA_ARAMBHA` | Gosaikunda Snan Begins | गोसाइँकुण्ड स्नान आरम्भ |  | NEPAL_PANCHANGA |
| `GOSAIKUNDA_SNANA_SAMAPTI` | Gosaikunda Snan Ends | गोसाइँकुण्ड स्नान समाप्ति |  | NEPAL_PANCHANGA |
| `HARIBODHINI_EKADASHI` | Haribodhini Ekadashi | हरिबोधिनी एकादशी |  | NEPAL_PANCHANGA |
| `HARITALIKA` | Haritalika Teej | हरितालिका तीज |  | NEPAL |
| `HATYAMOCHANA_SNANA` | Hatyamochan Snan | हत्यामोचन स्नान |  | NEPAL_PANCHANGA |
| `HILE_JATRA` | Hile Jatra | हिले जात्रा |  | NEPAL_PANCHANGA |
| `HOLI_HILLS` | Holi (Hills) | होली (पहाड) |  | NEPAL |
| `HOLI_TERAI` | Holi (Terai) | होली (तराई) |  | NEPAL |
| `JANMASHTAMI` | Krishna Janmashtami | कृष्ण जन्माष्टमी |  | NEPAL; excluded by the consumer at 12 of 14 (C616) |
| `KALI_YUGADI` | Kaliyugadi | कलियुगादि |  | NEPAL_PANCHANGA |
| `KUCHHI_BHOYE` | Kuchhi Bhoye | कुछिभ्वय् | Newar | NEPAL_PANCHANGA |
| `KUMARA_JATRA` | Kumar Jatra | कुमार जात्रा | Newar | NEPAL_PANCHANGA |
| `KURMA_JAYANTI` | Kurma Jayanti | कूर्म जयन्ती |  | NEPAL_PANCHANGA |
| `LAI_CHARHE` | Lai Charhe | लै चह्रे | Newar | NEPAL_PANCHANGA |
| `LAKSHMI_PUJA` | Laxmi Puja | लक्ष्मी पूजा |  | NEPAL |
| `LAXMI_PRASAD_DEVKOTA_JAYANTI` | Laxmi Prasad Devkota Jayanti | महाकवि लक्ष्मीप्रसाद देवकोटा जयन्ती |  | NEPAL_PANCHANGA |
| `MATSYANARAYANA_MELA` | Machhenarayan Mela | मच्छेनारायण मेला |  | NEPAL_PANCHANGA |
| `MATSYA_JAYANTI` | Matsya Jayanti | मत्स्य जयन्ती |  | NEPAL_PANCHANGA |
| `MHA_PUJA` | Mha Puja | म्हः पूजा | Newar | NEPAL_PANCHANGA |
| `NAVAMI_SHRADDHA` | Navami Shraddha | नवमी श्राद्ध |  | NEPAL_PANCHANGA |
| `NAVARATRA_ARAMBHA` | Ghatasthapana | घटस्थापना |  | NEPAL |
| `NEPAL_SAMBAT_NEW_YEAR` | Nepal Sambat New Year | नेपाल संवत् नयाँ वर्ष | Newar | NEPAL_PANCHANGA |
| `NIRJALA_EKADASHI` | Nirjala Ekadashi | निर्जला एकादशी |  | NEPAL_PANCHANGA |
| `NUWAKOT_DUPCHESHWAR_MELA` | Nuwakot Dupcheshwar Mela | नुवाकोट दुप्चेश्वर मेला |  | NEPAL_PANCHANGA |
| `PAHAN_CHARHE` | Pahan Charhe | पाहाँ चह्रे | Newar | NEPAL_PANCHANGA |
| `PANCHAMI_SHRADDHA` | Panchami Shraddha | पञ्चमी श्राद्ध |  | NEPAL_PANCHANGA |
| `PARASHURAMA_JAYANTI` | Parashuram Jayanti | परशुराम जयन्ती |  | NEPAL_PANCHANGA |
| `PASHUPATI_HALESI_MELA` | Pashupatinath and Halesi Mahadev Mela | पशुपतिनाथ तथा हलेसी महादेव मेला |  | NEPAL_PANCHANGA |
| `PAUSHA_PUTRADA_EKADASHI` | Paush Putrada Ekadashi | पौष पुत्रदा एकादशी |  | NEPAL_PANCHANGA |
| `PISHACHAMOCHANA_SHRADDHA` | Pishachmochan Shraddha | पिशाचमोचन श्राद्ध |  | NEPAL_PANCHANGA |
| `PRATIPADA_SHRADDHA` | Pratipada Shraddha | प्रतिपदा श्राद्ध |  | NEPAL_PANCHANGA |
| `RAKSHABANDHAN` | Raksha Bandhan | रक्षाबन्धन |  | NEPAL |
| `RAMA_NAVAMI` | Ram Navami | राम नवमी |  | NEPAL |
| `RISHI_PANCHAMI` | Rishi Panchami | ऋषिपञ्चमी |  | NEPAL_PANCHANGA |
| `SAPTAMI_SHRADDHA` | Saptami Shraddha | सप्तमी श्राद्ध |  | NEPAL_PANCHANGA |
| `SETO_MACHHINDRANATH_JATRA` | Seto Machhindranath Rath Jatra | सेतो मच्छिन्द्रनाथ रथ जात्रा | Newar | NEPAL_PANCHANGA |
| `SHANKARACHARYA_JAYANTI` | Shankaracharya Jayanti | शङ्कराचार्य जयन्ती |  | NEPAL_PANCHANGA |
| `SHASHTHI_SHRADDHA` | Shashthi Shraddha | षष्ठी श्राद्ध |  | NEPAL_PANCHANGA |
| `SHILA_CHARHE` | Sila Charhe | सिला चह्रे | Newar | NEPAL_PANCHANGA |
| `SHIVARATRI` | Maha Shivaratri | महाशिवरात्रि |  | NEPAL |
| `SITA_NAVAMI` | Sita Jayanti | सीता जयन्ती |  | NEPAL_PANCHANGA |
| `SORHA_SHRADDHA_ARAMBHA` | Sorha Shraddha Begins | सोह्र श्राद्ध आरम्भ |  | NEPAL_PANCHANGA |
| `TILAKUNDA_CHATURTHI` | Tilkunda Chaturthi | तिलकुन्द चतुर्थी |  | NEPAL_PANCHANGA |
| `TRAYODASHI_SHRADDHA` | Trayodashi Shraddha | त्रयोदशी श्राद्ध |  | NEPAL_PANCHANGA |
| `TRITIYA_SHRADDHA` | Tritiya Shraddha | तृतीया श्राद्ध |  | NEPAL_PANCHANGA |
| `TULASI_VIVAHA` | Tulsi Vivah | तुलसी विवाह |  | NEPAL_PANCHANGA |
| `UPAKARMA_MADHYANDINA` | Janai Purnima | जनै पूर्णिमा |  | NEPAL |
| `UPANGA_LALITA_VRATA` | Upanga Lalita Vrat | उपाङ्ग ललिता व्रत |  | NEPAL_PANCHANGA |
| `VAISHAKHA_SNANA_ARAMBHA` | Baisakh Snan Begins | वैशाख स्नान आरम्भ |  | NEPAL_PANCHANGA |
| `VAMANA_DWADASHI` | Vamana Dwadashi | वामन द्वादशी |  | NEPAL_PANCHANGA |
| `VIJAYA_DASHAMI` | Bijaya Dashami | विजया दशमी |  | NEPAL |
| `VIJAYA_EKADASHI` | Vijaya Ekadashi | विजया एकादशी |  | NEPAL_PANCHANGA |
| `YAMAPANCHAKA_ARAMBHA` | Yamapanchak Begins | यमपञ्चक आरम्भ |  | NEPAL_PANCHANGA |
| `YAMA_DWITIYA` | Bhai Tika | भाइटीका |  | NEPAL |

### Rule, not yet shipped (173)

The keys are proposed: each is a tithi in a month that the consumer's source reproduces at sunrise. A pack would carry each as an udaya rule after the back-test §10 of `festival-rules.md` gives the fitted rules.

| key | name | नाम | community | note |
|---|---|---|---|---|
| `ACHALA_SAPTAMI` | Achala Saptami | अचला सप्तमी |  |  |
| `ADUKHA_NAVAMI` | Adukha Navami | अदुःख नवमी | Newar |  |
| `AKHILBALI_PURTI` | Akhilbali Purti | अखिलबलि पूर्ति |  |  |
| `AKSHAYA_TRITIYA` | Akshaya Tritiya | अक्षय तृतीया |  |  |
| `AMALAKI_EKADASHI` | Amalaki Ekadashi | आमलकी एकादशी |  |  |
| `ANANTA_CHATURDASHI` | Ananta Chaturdashi | अनन्त चतुर्दशी |  |  |
| `ANNAPURNA_JATRA` | Annapurna Jatra | अन्नपूर्ण जात्रा |  |  |
| `APARA_EKADASHI` | Apara Ekadashi | अपरा एकादशी |  |  |
| `ARGHA_SARAI_PARVA` | Argha Sarai Parva | अर्घा सराय पर्व |  |  |
| `ARYAGHAT_MADHAV_NARAYAN_MELA` | Aryaghat Madhav Narayan Mela | आर्यघाट माधवनारायण मेला |  |  |
| `ARYA_DIWAS` | Arya Diwas | आर्य दिवस |  |  |
| `BAISAKH_SNAN_ENDS` | Baisakh Snan Ends | वैशाख स्नान समाप्ति |  |  |
| `BAKHUMADA` | Bakhumada | बखुमद |  |  |
| `BALAJU_BAISDHARA_MELA` | Balaju Baisdhara Mela | बालाजु बाइसधारा मेला |  |  |
| `BALAMBU_MAHALAXMI_JATRA` | Balambu Mahalaxmi Jatra | बलम्बु महालक्ष्मी जात्रा | Newar |  |
| `BALA_CHATURDASHI_BALA` | Bala Chaturdashi | बाला चतुर्दशी |  |  |
| `BASANTA_PANCHAMI` | Basanta Panchami | वसन्त पञ्चमी |  |  |
| `BHAKTAPUR_CHANDI_BHAGAWATI_JATRA` | Bhaktapur Chandi Bhagawati Jatra | भक्तपुर चण्डी भगवती जात्रा | Newar |  |
| `BHAKTAPUR_SIDDHA_POKHARI_MELA` | Bhaktapur Siddha Pokhari Mela | भक्तपुर सिद्धपोखरी मेला |  |  |
| `BHAL_BHAL_ASHTAMI` | Bhal Bhal Ashtami | भलभल अष्टमी | Newar |  |
| `BHISHMASHTAMI` | Bhishmashtami | भीष्माष्टमी |  |  |
| `BHISHMA_DWADASHI` | Bhishma Dwadashi | भीष्म द्वादशी |  |  |
| `BILVA_NIMANTRAN` | Bilva Nimantran | बिल्व निमन्त्रण |  |  |
| `BIRUDA_PANCHAMI` | Biruda Panchami | बिरुडा पञ्चमी |  |  |
| `BISHALNAGAR_VAISHNAVI_DEVI_JATRA` | Bishalnagar Vaishnavi Devi Jatra | विशालनगर वैष्णवी देवी जात्रा | Newar |  |
| `BUDDHAVATAR` | Buddhavatar | बुद्धावतार |  |  |
| `BUDDHA_JAYANTI` | Buddha Jayanti | बुद्ध जयन्ती | Buddhist |  |
| `CHAALA` | Chaala | चाँल |  |  |
| `CHANDI_PURNIMA` | Chandi Purnima | चण्डी पूर्णिमा |  |  |
| `CHANDRA_SHASHTHI` | Chandra Shashthi | चन्द्र षष्ठी |  |  |
| `CHANGUNARAYAN_AKHANDA_DEEP_DARSHAN` | Changunarayan Akhanda Deep Darshan | चाँगुनारायण अखण्ड दीप दर्शन |  |  |
| `CHANGUNARAYAN_CHHINNAMASTA_KILESHWOR_MULRATH_JATRA` | Changunarayan Chhinnamasta Kileshwor Mulrath Jatra | चाँगुनारायण छिन्नमस्ता किलेश्वर मूलरथ जात्रा | Newar |  |
| `CHANGUNARAYAN_KALASH_JATRA` | Changunarayan Kalash Jatra | चाँगुनारायण कलश जात्रा | Newar |  |
| `CHANGU_MADHAV_NARAYAN_MELA` | Changu Madhav Narayan Mela | चाँगु माधवनारायण मेला |  |  |
| `CHATURMAS_BEGINS` | Chaturmas Begins | चतुर्मास आरम्भ |  |  |
| `CHATURMAS_ENDS` | Chaturmas Ends | चतुर्मास समाप्ति |  |  |
| `CHHATH` | Chhath | छठ पर्व | Maithil and Bhojpuri |  |
| `CHIR_SWAYEGU` | Chir Swayegu | चीर स्वायेगु | Newar |  |
| `CHONAM_PARVA` | Chonam Parva | चोनाम पर्व |  |  |
| `DAKSHINAMURTI_PUJA` | Dakshinamurti Puja | दक्षिणामूर्ति पूजा |  |  |
| `DAR_KHANE_DIN` | Dar Khane Din | दर खाने दिन |  |  |
| `DEOPATAN_GANGAMAI_RATH_JATRA` | Deopatan Gangamai Rath Jatra | देवपाटन गङ्गामाई रथ जात्रा | Newar |  |
| `DEOPATAN_TRISHUL_JATRA` | Deopatan Trishul Jatra | देवपाटन त्रिशूल जात्रा | Newar |  |
| `DEVI_VISARJAN` | Devi Visarjan | देवी विसर्जन |  |  |
| `DHANTERAS` | Dhanteras | धनतेरस |  |  |
| `DHANYA_PURNIMA` | Dhanya Purnima | धान्य पूर्णिमा |  |  |
| `DHAPAKHEL_NAGDAHA` | Dhapakhel Nagdaha | धापाखेल नागदह |  |  |
| `DILLA_PUNHI` | Dilla Punhi | दिल्ला पुन्ही | Newar |  |
| `DRONA_NAVAMI` | Drona Navami | द्रोण नवमी |  |  |
| `FULKHARKA_GANGAJAMUNA_MELA` | Fulkharka Gangajamuna Mela | फूलखर्क गङ्गाजमुना मेला |  |  |
| `GAI_JATRA` | Gai Jatra | गाईजात्रा | Newar |  |
| `GANGA_PUJA` | Ganga Puja | गङ्गा पूजा |  |  |
| `GATILA` | Gatila | गातिला |  |  |
| `GAURI_VRAT` | Gauri Vrat | गौरी व्रत |  |  |
| `GAYA_MAUNI_AUNSI` | Gaya Mauni Aunsi | गया मौनी औँसी |  |  |
| `GITA_JAYANTI` | Gita Jayanti | गीता जयन्ती |  |  |
| `GOPASHTAMI` | Gopashtami | गोपाष्टमी |  |  |
| `GORAKHNATH_JAYANTI` | Gorakhnath Jayanti | गोरखनाथ जयन्ती |  |  |
| `GOVARDHAN_PUJA` | Govardhan Puja | गोवर्धन पूजा |  |  |
| `GOVINDA_DWADASHI` | Govinda Dwadashi | गोविन्द द्वादशी |  |  |
| `GUGA_NAVAMI` | Guga Navami | गुगा नवमी |  |  |
| `GUHYESHWARI_JATRA` | Guhyeshwari Jatra | गुह्येश्वरी जात्रा | Newar |  |
| `GUNLA_BEGINS` | Gunla Begins | गुँला पर्व आरम्भ | Newar |  |
| `GUNLA_ENDS` | Gunla Ends | गुँला पर्व समाप्ति | Newar |  |
| `GURU_NANAK_JAYANTI` | Guru Nanak Jayanti | गुरु नानक जयन्ती | Sikh |  |
| `GURU_PADMASAMBHAVA_JAYANTI` | Guru Padmasambhava Jayanti | गुरु पद्मसम्भव जयन्ती | Buddhist |  |
| `GURU_PURNIMA` | Guru Purnima | गुरु पूर्णिमा |  |  |
| `GYALPO_LHOSAR` | Gyalpo Lhosar | ग्याल्पो ल्होसार | Sherpa |  |
| `HALI_TIHAR` | Hali Tihar | हलि तिहार |  |  |
| `HANUMAN_JAYANTI` | Hanuman Jayanti | हनुमान जयन्ती |  |  |
| `HARIPARIVARTINI_EKADASHI` | Hariparivartini Ekadashi | हरिपरिवर्तिनी एकादशी |  |  |
| `HARISHAYANI_EKADASHI` | Harishayani Ekadashi | हरिशयनी एकादशी |  |  |
| `HETAUDA_KUSHMANDA_SAROBAR_MELA` | Hetauda Kushmanda Sarobar Mela | हेटौडा कुष्माण्ड सरोवर मेला |  |  |
| `INDIRA_EKADASHI` | Indira Ekadashi | इन्दिरा एकादशी |  |  |
| `INDRADAHA_SNAN` | Indradaha Snan | इन्द्रदह स्नान | Newar |  |
| `INDRADHWAJA_UTTHAN` | Indradhwaja Utthan | इन्द्रध्वज उत्थान | Newar |  |
| `INDRA_JATRA` | Indra Jatra | इन्द्रजात्रा | Newar |  |
| `JAGANNATH_RATH_JATRA` | Jagannath Rath Jatra | जगन्नाथ रथ जात्रा |  |  |
| `JAMARE_AUNSI` | Jamare Aunsi | जमरे औँसी |  |  |
| `JITIYA` | Jitiya | जितिया पर्व | Tharu and Maithil |  |
| `JYAPU_DIWAS` | Jyapu Diwas | ज्यापु दिवस | Newar |  |
| `JYA_PUNHI` | Jya Punhi | ज्या पुन्ही | Newar |  |
| `KABIR_JAYANTI` | Kabir Jayanti | कबीर जयन्ती |  |  |
| `KAG_TIHAR` | Kag Tihar | काग तिहार |  |  |
| `KAMIKA_EKADASHI` | Kamika Ekadashi | कामिका एकादशी |  |  |
| `KARTIK_SNAN_BEGINS` | Kartik Snan Begins | कार्तिक स्नान आरम्भ |  |  |
| `KARTIK_SNAN_ENDS` | Kartik Snan Ends | कार्तिक स्नान समाप्ति |  |  |
| `KATIN_PUNHI` | Katin Punhi | कतिं पुन्ही | Newar |  |
| `KAYA_ASHTAMI` | Kaya Ashtami | कायः अष्टमी | Newar |  |
| `KHADGA_JATRA` | Khadga Jatra | खड्ग जात्रा | Newar |  |
| `KHANCHI_SARAI_PARVA` | Khanchi Sarai Parva | खाँचीमा सराय पर्व |  |  |
| `KIRTIPUR_INDRAYANI_JATRA` | Kirtipur Indrayani Jatra | कीर्तिपुर इन्द्रायणी जात्रा | Newar |  |
| `KOKA_KAUSHIKI_SANGAM_MELA` | Koka Kaushiki Sangam Mela | कोका कौशिकी सङ्गम मेला |  |  |
| `KOKHAJA_BIYE` | Kokhaja Biye | कोखजा बिये |  |  |
| `KUKUR_TIHAR` | Kukur Tihar | कुकुर तिहार |  |  |
| `KUMAR_SHASHTHI` | Kumar Shashthi | कुमार षष्ठी |  |  |
| `KUSHE_AUNSI` | Kushe Aunsi | कुशे औँसी |  |  |
| `KUSHMANDA_NAVAMI` | Kushmanda Navami | कूष्माण्ड नवमी |  |  |
| `KWATI_PUNHI` | Kwati Punhi | क्वाति पुन्ही | Newar |  |
| `MADHAV_NARAYAN_MAHOTSAV` | Madhav Narayan Mahotsav | माधवनारायण महोत्सव |  |  |
| `MAGHA_SHRADDHA` | Magha Shraddha | मघा श्राद्ध |  |  |
| `MAGH_SNAN_BEGINS` | Magh Snan Begins | माघ स्नान आरम्भ |  |  |
| `MAGH_SNAN_ENDS` | Magh Snan Ends | माघ स्नान समाप्ति |  |  |
| `MAHAVIR_JAYANTI` | Mahavir Jayanti | महावीर जयन्ती | Jain |  |
| `MAHA_ASHTAMI` | Maha Ashtami | महाअष्टमी |  |  |
| `MAHA_NAVAMI` | Maha Navami | महानवमी |  |  |
| `MASTA_PURNIMA` | Masta Purnima | मष्ट पूर्णिमा |  |  |
| `MATA_TIRTHA_AUNSI` | Mata Tirtha Aunsi | मातातीर्थ औँसी |  |  |
| `MATINA_PARU` | Matina Paru | मतिना पारु | Newar |  |
| `MOHINI_EKADASHI` | Mohini Ekadashi | मोहिनी एकादशी |  |  |
| `MOKSHADA_EKADASHI` | Mokshada Ekadashi | मोक्षदा एकादशी |  |  |
| `MOTIRAM_JAYANTI` | Motiram Jayanti | मोतीराम जयन्ती |  |  |
| `MUKHA_ASHTAMI` | Mukha Ashtami | मुखः अष्टमी | Newar |  |
| `MUSYADULI` | Musyaduli | मुस्याडुली |  |  |
| `NAG_PANCHAMI` | Nag Panchami | नाग पञ्चमी |  |  |
| `NAKWA_DISHI` | Nakwa Dishi | नक्वः दिसि | Newar |  |
| `NALA_MACHHINDRANATH_RATH_JATRA` | Nala Machhindranath Rath Jatra | नाला मच्छिन्द्रनाथ रथ जात्रा | Newar |  |
| `NARAK_CHATURDASHI` | Narak Chaturdashi | नरक चतुर्दशी |  |  |
| `NEPALI_ARMY_DAY` | Nepali Army Day | नेपाली सेना दिवस |  |  |
| `PACHALI_BHAIRAV_JATRA` | Pachali Bhairav Jatra | पचली भैरव जात्रा | Newar |  |
| `PANGA_VAISHNAVI_DEVI_JATRA` | Panga Vaishnavi Devi Jatra | पाँगा वैष्णवी देवी जात्रा | Newar |  |
| `PAPANKUSHA_EKADASHI` | Papankusha Ekadashi | पापाङ्कुशा एकादशी |  |  |
| `PAPMOCHANI_EKADASHI` | Papmochani Ekadashi | पापमोचनी एकादशी |  |  |
| `PASHUPATINATH_DAMAN_PUSHPARPAN` | Pashupatinath Daman Pushparpan | पशुपतिनाथमा दमन पुष्पार्पण |  |  |
| `PHULPATI` | Phulpati | फूलपाती |  |  |
| `PUNCHALI_BHUJA` | Punchali Bhuja | पुञ्चली भुजा |  |  |
| `RADHASHTAMI` | Radhashtami | राधाष्टमी |  |  |
| `RAMA_EKADASHI` | Rama Ekadashi | रमा एकादशी |  |  |
| `RAMESHWAR_PUJA` | Rameshwar Puja | रामेश्वर पूजा |  |  |
| `RATO_MACHHINDRANATH_RATHAROHAN` | Rato Machhindranath Ratharohan | रातो मच्छिन्द्रनाथ रथारोहण | Newar |  |
| `RATO_MACHHINDRANATH_RATH_JATRA_BEGINS` | Rato Machhindranath Rath Jatra Begins | रातो मच्छिन्द्रनाथ रथ जात्रा आरम्भ | Newar |  |
| `ROPAI_JATRA` | Ropai Jatra | रोपाइँ जात्रा |  |  |
| `SAFALA_EKADASHI` | Safala Ekadashi | सफला एकादशी |  |  |
| `SAKWO_CHANGU_WONE` | Sakwo Changu Wone | सक्व चाँगु वने | Newar |  |
| `SAMVATSARA_ARAMBHA` | Samvatsara Arambha | संवत्सरारम्भ |  |  |
| `SANKHU_JATRA` | Sankhu Jatra | साँखु जात्रा | Newar |  |
| `SANSKRIT_DAY` | Sanskrit Day | संस्कृत दिवस |  |  |
| `SATYAYUGADI` | Satyayugadi | सत्ययुगादि |  |  |
| `SETO_MACHHINDRANATH_SNAN` | Seto Machhindranath Snan | सेतो मच्छिन्द्रनाथ स्नान | Newar |  |
| `SHADANANDA_JAYANTI` | Shadananda Jayanti | षडानन्द जयन्ती |  |  |
| `SHALYA_DASHAMI` | Shalya Dashami | शल्य दशमी |  |  |
| `SHANI_JAYANTI` | Shani Jayanti | शनि जयन्ती |  |  |
| `SHATTILA_EKADASHI` | Shattila Ekadashi | षट्तिला एकादशी |  |  |
| `SHITALASHTAMI` | Shitalashtami | शीतलाष्टमी |  |  |
| `SHIVA_BAIKUNTHA_CHATURDASHI` | Shiva Baikuntha Chaturdashi | शिव बैकुण्ठ चतुर्दशी |  |  |
| `SHRAWAN_PUTRADA_EKADASHI` | Shrawan Putrada Ekadashi | श्रावण पुत्रदा एकादशी |  |  |
| `SHREEGHA_JATRA` | Shreegha Jatra | श्रीघः जात्रा | Newar |  |
| `SHREE_BALLABH_JAYANTI` | Shree Ballabh Jayanti | श्री बल्लभ जयन्ती |  |  |
| `SILANGA_MAHARUDRA_KOTIHOM_MELA` | Silanga Maharudra Kotihom Mela | सिलङ्गा महारुद्र कोटिहोम मेला |  |  |
| `SITHI_NAKHA` | Sithi Nakha | सिथि नखः | Newar |  |
| `SKANDA_SHASHTHI` | Skanda Shashthi | स्कन्द षष्ठी |  |  |
| `SONAM_LHOSAR` | Sonam Lhosar | सोनाम ल्होसार | Tamang |  |
| `SURYA_PUJA` | Surya Puja | सूर्य पूजा |  |  |
| `SURYA_SHASHTHI` | Surya Shashthi | सूर्य षष्ठी |  |  |
| `SWASTHANI_VRAT_BEGINS` | Swasthani Vrat Begins | स्वस्थानी व्रत आरम्भ |  |  |
| `SWASTHANI_VRAT_ENDS` | Swasthani Vrat Ends | स्वस्थानी व्रत समाप्ति |  |  |
| `SWONTI_CHARHE` | Swonti Charhe | स्वन्ति चह्रे | Newar |  |
| `SYAKWA_TYAKWA` | Syakwa Tyakwa | स्याक्वत्याक्व | Newar |  |
| `THAKALI_PHALO_PARVA` | Thakali Phalo Parva | थकाली फालो पर्व | Thakali |  |
| `THANKOT_MAHALAXMI_JATRA` | Thankot Mahalaxmi Jatra | थानकोट महालक्ष्मी जात्रा | Newar |  |
| `TOL_LHOSAR` | Tol Lhosar | तोल ल्होसार |  |  |
| `TRETAYUGADI` | Tretayugadi | त्रेतायुगादि |  |  |
| `TULSIDAS_JAYANTI` | Tulsidas Jayanti | तुलसीदास जयन्ती |  |  |
| `UBHAULI` | Ubhauli | उभौली | Kirat |  |
| `UDHAULI` | Udhauli | उँधौली | Kirat |  |
| `UTPATTIKA_EKADASHI` | Utpattika Ekadashi | उत्पत्तिका एकादशी |  |  |
| `VARUTHINI_EKADASHI` | Varuthini Ekadashi | वरुथिनी एकादशी |  |  |
| `VAYU_ASHTAMI` | Vayu Ashtami | वायु अष्टमी |  |  |
| `VIVAH_PANCHAMI` | Vivah Panchami | विवाह पञ्चमी |  |  |
| `WATU_MAGH_JATRA` | Watu Magh Jatra | वटु माघ जात्रा | Newar |  |
| `YALA_PANCHADAN` | Yala Panchadan | यल पञ्चदान | Newar |  |
| `YOGINI_EKADASHI` | Yogini Ekadashi | योगिनी एकादशी |  |  |
| `YOMARI_PUNHI` | Yomari Punhi | योमरी पुन्ही | Newar |  |

### Rule, needs a form (43)

`festival-rule-language.md` §4 maps each to the form it needs, or says why none fits.

| key | name | नाम | community | note |
|---|---|---|---|---|
| `ARJUNDHARA_MELA` | Arjundhara Mela | अर्जुनधारा मेला |  |  |
| `BAHIDYO_BOYEGU` | Bahidyo Boyegu | बहीद्यः बोयेगु | Newar |  |
| `DEOPATAN_DESHODDHAR_PUJA` | Deopatan Deshoddhar Puja | देवपाटन देशोद्धार पूजा |  |  |
| `FAGU_PURNIMA` | Fagu Purnima | फागु पूर्णिमा |  |  |
| `GAI_TIHAR` | Gai Tihar | गाई तिहार |  |  |
| `GATHAMUGA_CHARHE` | Gathamuga Charhe | गथाँमुग चह्रे | Newar |  |
| `GAURA_PARVA` | Gaura Parva | गौरा पर्व |  |  |
| `GHANTAKARNA_CHATURDASHI` | Ghantakarna Chaturdashi | घण्टाकर्ण चतुर्दशी |  |  |
| `GOVATSA_DWADASHI` | Govatsa Dwadashi | गोवत्स द्वादशी |  |  |
| `HAYAGRIVA_JAYANTI` | Hayagriva Jayanti | हयग्रीव जयन्ती |  |  |
| `INDRADHWAJA_PATAN` | Indradhwaja Patan | इन्द्रध्वज पातन | Newar |  |
| `JUGA_CHARHE` | Juga Charhe | जुगः चह्रे | Newar |  |
| `KALKI_JAYANTI` | Kalki Jayanti | कल्कि जयन्ती |  |  |
| `KAMADA_EKADASHI` | Kamada Ekadashi | कामदा एकादशी |  |  |
| `KASHI_BADA_GANESH_UTSAV` | Kashi Bada Ganesh Utsav | काशी बडागणेश उत्सव |  |  |
| `KOJAGRAT_PURNIMA` | Kojagrat Purnima | कोजाग्रत पूर्णिमा |  |  |
| `KRISHNA_RATH_JATRA` | Krishna Rath Jatra | कृष्ण रथ जात्रा |  |  |
| `LALITPUR_NARSINGH_JATRA` | Lalitpur Narsingh Jatra | ललितपुर नृसिंह जात्रा | Newar |  |
| `MAHALAXMI_VRAT_BEGINS` | Mahalaxmi Vrat Begins | महालक्ष्मी व्रत आरम्भ |  |  |
| `MAHALAXMI_VRAT_ENDS` | Mahalaxmi Vrat Ends | महालक्ष्मी व्रत समाप्ति |  |  |
| `MAJHI_LADI_PUJA` | Majhi Ladi Puja | माझी लदी पूजा | Majhi |  |
| `MATATI_CHARHE` | Matati Charhe | माताति चह्रे | Newar |  |
| `MATSYA_DARSHAN` | Matsya Darshan | मत्स्य दर्शन |  |  |
| `NALASWANE_CHARHE` | Nalaswane Charhe | नःलास्वने चह्रे | Newar |  |
| `NALA_MACHHINDRANATH_SNAN` | Nala Machhindranath Snan | नाला मच्छिन्द्रनाथ स्नान | Newar |  |
| `NANICHAYA` | Nanichaya | नानिचाया |  |  |
| `NARASIMHA_JAYANTI` | Narasimha Jayanti | नृसिंह जयन्ती |  |  |
| `PADMINI_EKADASHI` | Padmini Ekadashi | पद्मिनी एकादशी |  | the adhika month only (§3.7) |
| `PANAUTI_JATRA` | Panauti Jatra | पनौती जात्रा | Newar | both editions print it on Jyeshtha's full moon |
| `PANAUTI_SNAN_MELA` | Panauti Snan Mela | पनौती स्नान मेला |  | both editions print it on Jyeshtha's full moon |
| `PARAMA_EKADASHI` | Parama Ekadashi | परमा एकादशी |  | the adhika month only (§3.7) |
| `PASHUPATINATH_CHHAYA_DARSHAN` | Pashupatinath Chhaya Darshan | पशुपतिनाथ छाया दर्शन |  |  |
| `PHARPING_HARISHANKAR_JATRA` | Pharping Harishankar Jatra | फर्पिङ हरिशङ्कर जात्रा |  |  |
| `RADHA_KRISHNA_RATH_JATRA` | Radha Krishna Rath Jatra | राधाकृष्ण रथ जात्रा |  |  |
| `RAMANUJACHARYA_JAYANTI` | Ramanujacharya Jayanti | रामानुजाचार्य जयन्ती |  |  |
| `RATO_MACHHINDRANATH_SNAN` | Rato Machhindranath Snan | रातो मच्छिन्द्रनाथ स्नान | Newar |  |
| `SHIVA_PAVITRAROPAN` | Shiva Pavitraropan | शिव पवित्रारोपण |  |  |
| `SITHI_CHARHE` | Sithi Charhe | सिथि चह्रे | Newar |  |
| `SI_PUNHI` | Si Punhi | सिपुन्ही | Newar |  |
| `VARAHA_JAYANTI` | Varaha Jayanti | वराह जयन्ती |  |  |
| `VAT_SAVITRI_VRAT` | Vat Savitri Vrat | वटसावित्री व्रत |  |  |
| `VISHNU_BAIKUNTHA_CHATURDASHI` | Vishnu Baikuntha Chaturdashi | विष्णु बैकुण्ठ चतुर्दशी |  |  |
| `WALKE_PARVA` | Walke Parva | वल्के पर्व |  | BS 5/1 or Shravana bright 13th, 2 of 3 |

### Solar (26)

A day of the solar month (C619), as `SOLAR_DAY` would say it (`festival-rule-language.md` §3.6).

| key | name | नाम | community | note |
|---|---|---|---|---|
| `BHAKTAPUR_BRAHMAYANI_JATRA` | Bhaktapur Brahmayani Jatra | भक्तपुर ब्रह्मायणी जात्रा | Newar |  |
| `BHAKTAPUR_CHHUMA_GANESH_JATRA` | Bhaktapur Chhuma Ganesh Jatra | भक्तपुर छुमा गणेश जात्रा | Newar |  |
| `BHAKTAPUR_MAHAKALI_MAHALAXMI_JATRA` | Bhaktapur Mahakali Mahalaxmi Jatra | भक्तपुर महाकाली महालक्ष्मी जात्रा | Newar |  |
| `BHUMIRAJ` | Bhumiraj | भूमिरज |  |  |
| `BHUMI_PUJA` | Bhumi Puja | भूमि पूजा |  |  |
| `BISKET_JATRA_BEGINS` | Bisket Jatra Begins | बिस्केट जात्रा आरम्भ | Newar |  |
| `BISKET_JATRA_ENDS` | Bisket Jatra Ends | बिस्केट जात्रा समाप्ति | Newar |  |
| `BODE_JIBRO_CHHEDNE_JATRA` | Bode Jibro Chhedne Jatra | बोडे जिब्रो छेड्ने जात्रा | Newar |  |
| `DEVGHAT_MAKAR_SNAN` | Devghat Makar Snan | देवघाट मकर स्नान |  |  |
| `GURIYA_PARVA` | Guriya Parva | गुरिया पर्व | Tharu |  |
| `KANDAKARAK_PUJA` | Kandakarak Puja | कण्डकारक पूजा |  |  |
| `KHEER_KHANE_DIN` | Kheer Khane Din | खीर खाने दिन |  |  |
| `KIRTIPUR_BAGH_BHAIRAV_JATRA` | Kirtipur Bagh Bhairav Jatra | कीर्तिपुर बाघभैरव जात्रा | Newar |  |
| `MAGHE_SANKRANTI` | Maghe Sankranti | माघे सङ्क्रान्ति |  |  |
| `MAGHI_THARU` | Maghi (Tharu) | माघी | Tharu |  |
| `MAKAR_SNAN` | Makar Snan | मकर स्नान |  |  |
| `NEPALI_NEW_YEAR` | Nepali New Year | नव वर्ष |  |  |
| `SINDOOR_JATRA` | Sindoor Jatra | सिन्दूर जात्रा | Newar |  |
| `SIRUWA` | Siruwa | सिरुवा पर्व | Maithil, Tharu and Rajbanshi |  |
| `TAMU_LHOSAR` | Tamu Lhosar | तमु ल्होसार | Gurung |  |
| `THIMI_BALKUMARI_JATRA` | Thimi Balkumari Jatra | ठिमी बालकुमारी जात्रा | Newar |  |
| `TOKHA_CHANDESHWARI_JATRA` | Tokha Chandeshwari Jatra | टोखा चण्डेश्वरी जात्रा | Newar |  |
| `TOKHA_SAPAN_TIRTHA_SNAN` | Tokha Sapan Tirtha Snan | टोखा सपनतीर्थ स्नान |  |  |
| `VISHWAKARMA_PUJA` | Vishwakarma Puja | विश्वकर्मा पूजा |  |  |
| `YOSIN_PATAN` | Yosin Patan | योसिं पतन | Newar |  |
| `YOSIN_UTTHAN` | Yosin Utthan | योसिं उत्थान | Newar |  |

### Community (6)

No rule is in hand; the community sets the day.

| key | name | नाम | community | note |
|---|---|---|---|---|
| `BABA_GANINATH_JAYANTI` | Baba Ganinath Jayanti | बाबा गणिनाथ जयन्ती | Kanu (Madhesh) |  |
| `BHOTO_JATRA` | Bhoto Jatra | भोटो जात्रा | Newar |  |
| `DISHI_CHARHE` | Dishi Charhe | दिशी चह्रे | Newar |  |
| `DISHI_PUJA` | Dishi Puja | दिशी पूजा | Newar |  |
| `DURA_MHAIPRU_NAKUMA` | Dura Mhaipru Nakuma | दुरा म्हैप्रु नकुमा |  |  |
| `THAKALI_TORANLA_PARVA` | Thakali Toranla Parva | थकाली तोरन्ला पर्व | Thakali |  |

### Published only (5)

An authority announces the day.

| key | name | नाम | community | note |
|---|---|---|---|---|
| `BARAHAKSHETRA_MELA` | Barahakshetra Mela | वराहक्षेत्र मेला |  |  |
| `EID_AL_ADHA` | Eid al-Adha | बकर ईद | Muslim |  |
| `EID_AL_FITR` | Eid al-Fitr | ईद उल फित्र | Muslim |  |
| `MAWLID` | Mawlid | मोहम्मद जयन्ती | Muslim |  |
| `MUKTAK_DIWAS` | Muktak Diwas | मुक्तक दिवस |  |  |

### Fixed civil (165)

A date in the Bikram Sambat or the Gregorian calendar, fixed by law or by the body that keeps it. These are listed by key and English name only.

| key | name |
|---|---|
| `APRIL_FOOLS_DAY` | April Fools' Day |
| `ARANIKO_MEMORIAL_DAY` | Araniko Memorial Day |
| `BHANU_JAYANTI` | Bhanu Jayanti |
| `BP_SMRITI_DIWAS` | BP Smriti Diwas |
| `CASTE_BASED_DISCRIMINATION_AND_UNTOUCHABILITY_ELIMINATION_DAY` | Caste-based Discrimination and Untouchability Elimination Day |
| `CHHANDA_DIWAS` | Chhanda Diwas |
| `CHRISTMAS` | Christmas |
| `CIVIL_SERVICE_DAY` | Civil Service Day |
| `CONSTITUTION_DAY` | Constitution Day |
| `DEMOCRACY_DAY` | Democracy Day |
| `EARTHQUAKE_MEMORIAL_DAY` | Earthquake Memorial Day |
| `EARTHQUAKE_SAFETY_DAY` | Earthquake Safety Day |
| `EARTH_DAY` | Earth Day |
| `ELECTION_DAY` | Election Day |
| `FALGUNANDA_JAYANTI` | Falgunanda Jayanti |
| `FOREST_USERS_DAY` | Forest Users' Day |
| `GEN_Z_MARTYRS_DAY` | Gen Z Martyrs' Day |
| `GLOBAL_DAY_OF_PARENTS` | Global Day of Parents |
| `GLOBAL_HANDWASHING_DAY` | Global Handwashing Day |
| `GURU_GOBIND_SINGH_JAYANTI` | Guru Gobind Singh Jayanti |
| `HUMAN_RIGHTS_DAY` | Human Rights Day |
| `INTERNATIONAL_ANTI_CORRUPTION_DAY` | International Anti-Corruption Day |
| `INTERNATIONAL_CHILDHOOD_CANCER_DAY` | International Childhood Cancer Day |
| `INTERNATIONAL_CIVIL_AVIATION_DAY` | International Civil Aviation Day |
| `INTERNATIONAL_CUSTOMS_DAY` | International Customs Day |
| `INTERNATIONAL_DANCE_DAY` | International Dance Day |
| `INTERNATIONAL_DAY_AGAINST_DRUG_ABUSE_AND_ILLICIT_TRAFFICKING` | International Day against Drug Abuse and Illicit Trafficking |
| `INTERNATIONAL_DAY_FOR_BIOLOGICAL_DIVERSITY` | International Day for Biological Diversity |
| `INTERNATIONAL_DAY_FOR_DISASTER_RISK_REDUCTION` | International Day for Disaster Risk Reduction |
| `INTERNATIONAL_DAY_FOR_THE_ABOLITION_OF_SLAVERY` | International Day for the Abolition of Slavery |
| `INTERNATIONAL_DAY_FOR_THE_ELIMINATION_OF_RACIAL_DISCRIMINATION` | International Day for the Elimination of Racial Discrimination |
| `INTERNATIONAL_DAY_FOR_THE_ELIMINATION_OF_VIOLENCE_AGAINST_WOMEN` | International Day for the Elimination of Violence against Women |
| `INTERNATIONAL_DAY_FOR_THE_ERADICATION_OF_POVERTY` | International Day for the Eradication of Poverty |
| `INTERNATIONAL_DAY_FOR_THE_TOTAL_ELIMINATION_OF_NUCLEAR_WEAPONS` | International Day for the Total Elimination of Nuclear Weapons |
| `INTERNATIONAL_DAY_FOR_TOLERANCE` | International Day for Tolerance |
| `INTERNATIONAL_DAY_OF_ACTION_FOR_WOMEN_S_HEALTH` | International Day of Action for Women's Health |
| `INTERNATIONAL_DAY_OF_EDUCATION` | International Day of Education |
| `INTERNATIONAL_DAY_OF_FAMILIES` | International Day of Families |
| `INTERNATIONAL_DAY_OF_HAPPINESS` | International Day of Happiness |
| `INTERNATIONAL_DAY_OF_NON_VIOLENCE` | International Day of Non-Violence |
| `INTERNATIONAL_DAY_OF_OLDER_PERSONS` | International Day of Older Persons |
| `INTERNATIONAL_DAY_OF_PEACE` | International Day of Peace |
| `INTERNATIONAL_DAY_OF_PERSONS_WITH_DISABILITIES` | International Day of Persons with Disabilities |
| `INTERNATIONAL_DAY_OF_RURAL_WOMEN` | International Day of Rural Women |
| `INTERNATIONAL_DAY_OF_SIGN_LANGUAGES` | International Day of Sign Languages |
| `INTERNATIONAL_DAY_OF_SPORT_FOR_DEVELOPMENT_AND_PEACE` | International Day of Sport for Development and Peace |
| `INTERNATIONAL_DAY_OF_THE_WORLD_S_INDIGENOUS_PEOPLES` | International Day of the World's Indigenous Peoples |
| `INTERNATIONAL_DAY_OF_UNITED_NATIONS_PEACEKEEPERS` | International Day of United Nations Peacekeepers |
| `INTERNATIONAL_DAY_OF_YOGA` | International Day of Yoga |
| `INTERNATIONAL_LABOUR_DAY` | International Labour Day |
| `INTERNATIONAL_LITERACY_DAY` | International Literacy Day |
| `INTERNATIONAL_MIGRANTS_DAY` | International Migrants Day |
| `INTERNATIONAL_MOTHER_LANGUAGE_DAY` | International Mother Language Day |
| `INTERNATIONAL_MOUNTAIN_DAY` | International Mountain Day |
| `INTERNATIONAL_NURSES_DAY` | International Nurses Day |
| `INTERNATIONAL_OLYMPIC_DAY` | International Olympic Day |
| `INTERNATIONAL_SPORTS_JOURNALISTS_DAY` | International Sports Journalists Day |
| `INTERNATIONAL_STUDENTS_DAY` | International Students' Day |
| `INTERNATIONAL_VOLUNTEER_DAY` | International Volunteer Day |
| `INTERNATIONAL_WELLNESS_DAY` | International Wellness Day |
| `INTERNATIONAL_WOMEN_S_DAY` | International Women's Day |
| `INTERNATIONAL_YOUTH_DAY` | International Youth Day |
| `JANA_YUDDHA_DIWAS` | Jana Yuddha Diwas |
| `KAMAIYA_MUKTI_DIWAS` | Kamaiya Mukti Diwas |
| `KIRAT_SAMAJ_SUDHAR_DIWAS` | Kirat Samaj Sudhar Diwas |
| `LANGUAGE_DAY` | Language Day |
| `LAW_DAY` | Law Day |
| `LEKHNATH_PAUDYAL_JAYANTI` | Lekhnath Paudyal Jayanti |
| `LOKTANTRA_DIWAS` | Loktantra Diwas |
| `MADAN_ASHRIT_SMRITI_DIWAS` | Madan-Ashrit Smriti Diwas |
| `MARTYRS_DAY` | Martyrs' Day |
| `MICRO_SMALL_AND_MEDIUM_SIZED_ENTERPRISES_DAY` | Micro-, Small and Medium-sized Enterprises Day |
| `NATIONAL_BHAKKA_DAY` | National Bhakka Day |
| `NATIONAL_CHILDREN_S_DAY` | National Children's Day |
| `NATIONAL_DAY_AGAINST_HUMAN_TRAFFICKING` | National Day Against Human Trafficking |
| `NATIONAL_DRESS_DAY` | National Dress Day |
| `NATIONAL_FILM_DAY` | National Film Day |
| `NATIONAL_FLAG_DAY` | National Flag Day |
| `NATIONAL_JOURNALISM_DAY` | National Journalism Day |
| `NATIONAL_MAGAR_DAY` | National Magar Day |
| `NATIONAL_PADDY_DAY` | National Paddy Day |
| `NATIONAL_RECONCILIATION_DAY` | National Reconciliation Day |
| `NATIONAL_SCIENCE_DAY` | National Science Day |
| `NATIONAL_TEA_DAY` | National Tea Day |
| `NATIONAL_TOPI_DAY` | National Topi Day |
| `NATIONAL_YOGA_DAY` | National Yoga Day |
| `NEPAL_NURSING_DAY` | Nepal Nursing Day |
| `NEW_YEAR_S_DAY_AD` | New Year's Day (AD) |
| `NON_RESIDENT_NEPALI_DAY` | Non-Resident Nepali Day |
| `PASANG_LHAMU_SMRITI_DIWAS` | Pasang Lhamu Smriti Diwas |
| `PENSION_DAY` | Pension Day |
| `PI_DAY` | Pi Day |
| `PRITHVI_JAYANTI` | Prithvi Jayanti |
| `RASTRIYA_DHARMA_SABHA_DIWAS` | Rastriya Dharma Sabha Diwas |
| `REPUBLIC_DAY` | Republic Day |
| `SAFER_INTERNET_DAY` | Safer Internet Day |
| `SAGARMATHA_DAY` | Sagarmatha Day |
| `SIDDHICHARAN_SHRESTHA_JAYANTI` | Siddhicharan Shrestha Jayanti |
| `SOCIAL_SERVICE_DAY` | Social Service Day |
| `SWAMI_PARAMANANDA_MEMORIAL_DAY` | Swami Paramananda Memorial Day |
| `SWAMI_SHASHIDHAR_JAYANTI` | Swami Shashidhar Jayanti |
| `UNESCO_DAY` | UNESCO Day |
| `UNITED_NATIONS_DAY` | United Nations Day |
| `VALENTINE_S_DAY` | Valentine's Day |
| `VASTU_DIWAS` | Vastu Diwas |
| `WORLD_AIDS_DAY` | World AIDS Day |
| `WORLD_ALZHEIMER_S_DAY` | World Alzheimer's Day |
| `WORLD_ANIMAL_DAY` | World Animal Day |
| `WORLD_AUTISM_AWARENESS_DAY` | World Autism Awareness Day |
| `WORLD_BEE_DAY` | World Bee Day |
| `WORLD_BICYCLE_DAY` | World Bicycle Day |
| `WORLD_BLOOD_DONOR_DAY` | World Blood Donor Day |
| `WORLD_BOOK_AND_COPYRIGHT_DAY` | World Book and Copyright Day |
| `WORLD_CANCER_DAY` | World Cancer Day |
| `WORLD_CHILDREN_S_DAY` | World Children's Day |
| `WORLD_CITIES_DAY` | World Cities Day |
| `WORLD_CONSUMER_RIGHTS_DAY` | World Consumer Rights Day |
| `WORLD_CULTURE_DAY` | World Culture Day |
| `WORLD_DAY_AGAINST_CHILD_LABOUR` | World Day Against Child Labour |
| `WORLD_DAY_OF_SOCIAL_JUSTICE` | World Day of Social Justice |
| `WORLD_DAY_TO_COMBAT_DESERTIFICATION_AND_DROUGHT` | World Day to Combat Desertification and Drought |
| `WORLD_DEVELOPMENT_INFORMATION_DAY` | World Development Information Day |
| `WORLD_DIABETES_DAY` | World Diabetes Day |
| `WORLD_ENVIRONMENT_DAY` | World Environment Day |
| `WORLD_FIRST_AID_DAY` | World First Aid Day |
| `WORLD_FOOD_DAY` | World Food Day |
| `WORLD_FREEDOM_DAY` | World Freedom Day |
| `WORLD_HEALTH_DAY` | World Health Day |
| `WORLD_HEART_DAY` | World Heart Day |
| `WORLD_HERITAGE_DAY` | World Heritage Day |
| `WORLD_HYPERTENSION_DAY` | World Hypertension Day |
| `WORLD_INTELLECTUAL_PROPERTY_DAY` | World Intellectual Property Day |
| `WORLD_MALARIA_DAY` | World Malaria Day |
| `WORLD_MENTAL_HEALTH_DAY` | World Mental Health Day |
| `WORLD_METEOROLOGICAL_DAY` | World Meteorological Day |
| `WORLD_MUSIC_DAY` | World Music Day |
| `WORLD_NO_TOBACCO_DAY` | World No Tobacco Day |
| `WORLD_OCEANS_DAY` | World Oceans Day |
| `WORLD_ORAL_HEALTH_DAY` | World Oral Health Day |
| `WORLD_PHARMACISTS_DAY` | World Pharmacists Day |
| `WORLD_PHOTOGRAPHY_DAY` | World Photography Day |
| `WORLD_PHYSIOTHERAPY_DAY` | World Physiotherapy Day |
| `WORLD_POPULATION_DAY` | World Population Day |
| `WORLD_POST_DAY` | World Post Day |
| `WORLD_PRESS_FREEDOM_DAY` | World Press Freedom Day |
| `WORLD_RABIES_DAY` | World Rabies Day |
| `WORLD_RADIOGRAPHY_DAY` | World Radiography Day |
| `WORLD_RADIO_DAY` | World Radio Day |
| `WORLD_RED_CROSS_DAY` | World Red Cross Day |
| `WORLD_REFUGEE_DAY` | World Refugee Day |
| `WORLD_REMEMBRANCE_AND_RECONCILIATION_DAY` | World Remembrance and Reconciliation Day |
| `WORLD_SIGHT_DAY` | World Sight Day |
| `WORLD_SMILE_DAY` | World Smile Day |
| `WORLD_SOIL_CONSERVATION_DAY` | World Soil Conservation Day |
| `WORLD_SOIL_DAY` | World Soil Day |
| `WORLD_STANDARDS_DAY` | World Standards Day |
| `WORLD_TEACHERS_DAY` | World Teachers' Day |
| `WORLD_TELECOMMUNICATION_AND_INFORMATION_SOCIETY_DAY` | World Telecommunication and Information Society Day |
| `WORLD_TELEVISION_DAY` | World Television Day |
| `WORLD_THEATRE_DAY` | World Theatre Day |
| `WORLD_TOILET_DAY` | World Toilet Day |
| `WORLD_TOURISM_DAY` | World Tourism Day |
| `WORLD_TUBERCULOSIS_DAY` | World Tuberculosis Day |
| `WORLD_WATER_DAY` | World Water Day |
| `WORLD_WETLANDS_DAY` | World Wetlands Day |

## 4. Communities

The festivals one community keeps, by community. A festival may count in
more than one class above; this table counts it once.

| community | festivals |
|---|---:|
| Newar | 83 |
| Kirat | 4 |
| Muslim | 3 |
| Tharu | 3 |
| Buddhist | 2 |
| Sikh | 2 |
| Thakali | 2 |
| Christian | 1 |
| Gurung | 1 |
| Jain | 1 |
| Kanu (Madhesh) | 1 |
| Magar | 1 |
| Maithil and Bhojpuri | 1 |
| Maithil, Tharu and Rajbanshi | 1 |
| Majhi | 1 |
| Sherpa | 1 |
| Tamang | 1 |
| Tharu and Maithil | 1 |

The Newar festivals are named by the words their names carry (charhe,
punhi, jatra in the valley's towns, and the Newar names of the year's
rites). A Newar festival whose name carries none of them is left blank.
Most Newar festivals are lunar and are reckoned like any other. The ones
that fit no rule are the charhe (C614).

## 5. The 97 names that are no festival

These names map to no festival in the consumer's list. Each is classed
here by its name, without the days the list gives it:

| class | what decides the day | names |
|---|---|---|
| none | a name the source left blank | 3 |
| computed, not a festival | the SDK's eclipse search | Khandagras Chandragrahan, Lunar Eclipse, Partial Lunar Eclipse during sunrise (Grastodit Khandagras Lunar Eclipse), khandagrash Solar Eclipse |
| fixed civil | an organisation's own date | Astrology Day, Establishment of Mahila Jyotis Sangh, Nepal Jyotish Parisad Anniversary, Religious Cultural Journalist Society Foundation Day, Spiritual Journalist Society Foundation Day, World Astrology Federation Foundation Day, World Hindu Federation Establishment Day |
| published only | a one-off event | 21 names: sports fixtures, film and theatre festivals, anniversaries, a contest, a gathering |
| published only | no rule in hand | Gorakhakali Pooja, Haalo Saro Garne, Halo Barne, Nishi Barne, Nisi Barne, Saubhagya Surpa Danam, Trijatalai Aadharya Dine, Tulasiko Bijaa  Ropne, Tulasipiye, Tulsi Ropan, Ya:si Mata Bigu |
| published only | set by the government each time | Dashain Holiday, Election Holiday, 2082, Elections 2082 for House of Representatives, Federal and Provincial Election 2079, Gen-Z Martyrs' Mourning Day, Local Election 2079, Public Holiday, Tihar Holiday |
| rule, needs a form | a tithi on a weekday (§3.5) | Bhaumastami, Budhashtami Barta, Mangal Chauthi Brata, Ravi Saptami Brat, Sombare Ausi |
| rule, needs a form | either paksha's 8th (§3.4) | Asthami Barta |
| rule, needs a form | pradosha on either paksha's 13th, and on a weekday (§3.4, §3.5) | Pradosh Brata, Shani Pradosh Brata, Som Pradosh Brata |
| rule, needs a form | the adhika month only (§3.7) | Padmini Ekadashi Brata |
| rule, needs a form | the moonrise of the dark 4th (§3.1) | Chandrodaya |
| rule, not yet shipped | a tithi (or nakshatra) of each month or of a named month, which the language holds today | Darsha Shraddha, Manwadi, SaraswatI avahan garne, Snan and Daan Ausi, Snandan Garne Ausi, Yamadip Danam |
| rule, shipped | the monthly full-moon fast | Purnima Brata |
| solar | a sankranti, or a custom of one | Applying oil and eating mango mujura, Applying oil and eating neem leaves, Asar Sankranti, Ashwin Sankranti, Baishakh Sankranti, Bhadra Sankranti, Chaitra Sankranti, Dakshinaya Start, Eating Neem's Leaves, Falgun Sankranti, Jeshta Sankranti, Kartik Sankranti, Luto Falne, Marga Sirsha Sankranti, Poush Sankranti, Shrawane Sankranti, Shree Suryako 1:57 baje mithuna ma jane, Taking an oil bath in the morning, Uttarayana Starts |

Two of these matter beyond their counts:

- The monthly fasts (pradosha, a tithi on a weekday, either paksha's 8th,
  the dark 4th's moonrise) are the commonest names here. They are what
  `festival-rule-language.md` §3.1, §3.4 and §3.5 are for.
- The twelve sankrantis are already found by the solar calendar's
  sankranti search. A festival kept on one needs only a rule that names
  it.

