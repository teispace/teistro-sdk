# Uttar Pradesh's public holidays

Status: `built`, 2026-11-12. SR-03, the tenth state set, after Maharashtra's
(`maharashtra-holidays.md`), West Bengal's (`west-bengal-holidays.md`), Kerala's
(`kerala-holidays.md`), Tamil Nadu's (`tamil-nadu-holidays.md`), Assam's
(`assam-holidays.md`), Karnataka's (`karnataka-holidays.md`), Telangana's
(`telangana-holidays.md`), Gujarat's (`gujarat-holidays.md`) and Andhra Pradesh's
(`andhra-pradesh-holidays.md`). Cruxes C1520 to C1529
(`01-research/feature-universe/19-verification-cruxes.md`). Measured in
`festival-measured.md` §20.

## 1. The source

The General Administration Section (सामान्य प्रशासन अनुभाग) of the Government
of Uttar Pradesh issues one notification (विज्ञप्ति) each October to December,
`No. <n>/तीन-<year>-39(2)/2016`, from Lucknow, for the year after. In it the
Governor declares the days public holidays, under the Government of India's
notification No. 20/25-56-Pub-I of 8 June 1957, and it holds:

- **Para 2, the public holidays** (सार्वजनिक अवकाश), some twenty-five a year,
  "under the Negotiable Instruments Act, 1881". Each row prints the Gregorian
  date, the Saka date, the Vikram Samvat tithi by the purnimanta month (as
  "भाद्रपद कृष्ण 08" for Janmashtami) and the weekday. Para 2 also says that
  days which fall together are not declared on separate days, so one row may
  name two.
- **Para 3, the restricted holidays** (निर्बन्धित अवकाश), some thirty a year,
  from which an employee chooses; the notification does not say how many.
- **The asterisk**: under each list, the days marked `*`, the Islamic days, "will
  be kept by the local sighting of the moon".
- **Para 4**, the banks' closing of accounts on 1 April; **para 5**, the days
  declared by executive order (Guru Gobind Singh Jayanti, Guru Tegh Bahadur's
  martyrdom day); and **para 6**, a district magistrate's up to three local
  holidays a year (C1527).

All eight years were read from the notification's
own pages (C1520):

| year | notification | copy read |
|---|---|---|
| 2019 | No. 12/2018/846/तीन-2018-39(2)/2016, 27 November 2018 | the electronic issue (`shasanadesh.up.nic.in`), as the state police's site keeps it: `uppolice.gov.in/pdf/Holiday2019.pdf` |
| 2020 | No. 680/तीन-2019-39(2)/2016, 4 November 2019 | the electronic issue, the state police's copy: `uppolice.gov.in/pdf/Holiday2020.pdf` |
| 2021 | No. 19/2020/912/तीन-2020-39(2)/2016, 11 December 2020 | the electronic issue, the state police's copy, after the Secretariat's calendar: `uppolice.gov.in/pdf/Holiday2021.pdf` |
| 2022 | No. 10/2021/514/तीन-2021-39(2)/2016, 20 October 2021 | the electronic issue, as an HR firm republished it: `promptpersonnel.com/wp-content/uploads/2021/12/Uttarpradesh-State-List-of-Holidays.pdf` |
| 2023 | No. 508/तीन-2022-39(2)/2016, 15 October 2022 | the signed scan: `datocms-assets.com/40521/1668666884-uttar-pradesh-state-list-of-holidays-2023.pdf` |
| 2024 | No. 528/तीन-2023-39(2)/2016, 4 December 2023 | the signed scan: `datocms-assets.com/40521/1707036755-uttar-pradesh-list-of-holidays-2024.pdf`, and its pages on `shasnadesh.com/2023/12/2024-up-government-holiday.html` |
| 2025 | No. 870/तीन-2024-39(2)/2016, 17 December 2024 | the signed scan: `simpliance.in/uploads/notifdocs/2481/uttar-pradesh-state-list-of-holidays-2025.pdf` |
| 2026 | No. 671/तीन-2025-39(2)/2016, 17 November 2025 | the *Uttar Pradesh Gazette*, Extraordinary, Legislative Supplement, Part 4, Section (Kh), 17 November 2025, as `sarkarifile.com` shared it (`drive.google.com/file/d/1NqRuRTyxpyVjFAvI3uRa9eM_B2nvVbdF`) |

The state's servers (`up.gov.in`, `shasanadesh.up.gov.in`, `upgad.up.gov.in`)
were not reachable from here. The request named the Personnel (Karmik)
Department; every notification read is the General Administration Section's.

## 2. The pack

`INDIA_UTTAR_PRADESH` holds both lists at any place. It is measured at Lucknow
(26.8467 N, 80.9462 E) on India's clock. Every day is kept in `["IN", "IN-UP"]`
(ISO 3166-2; `IN-UP` maps to the pack, SR-89) and is `GAZETTED`. A restricted
holiday is `optional`, as Andhra Pradesh's optional holidays are; Kashyap and
Nishadraj Jayanti and Vishwakarma Puja name `HINDU`, Christmas Eve and the Easter
days `CHRISTIAN`.

| holiday | key | form | the day's rule is cited to |
|---|---|---|---|
| Republic Day, Independence Day, Gandhi Jayanti, Christmas, Good Friday | as `INDIA_CENTRAL` | the central list's rule | the notification |
| Mahashivratri, Holi, Ram Navami, Mahavir Jayanti, Buddha Purnima, Janmashtami, Guru Nanak Jayanti | `SHIVARATRI`, `HOLI`, `RAMA_NAVAMI`, `MAHAVIR_JAYANTI`, `BUDDHA_JAYANTI`, `JANMASHTAMI`, `GURU_NANAK_JAYANTI` | the central list's rule; Holi the day after `HOLIKA` | *Dharmasindhu* and a printed panchanga, as the central list cites them |
| Dussehra (Vijayadashami), Deepawali | `VIJAYA_DASHAMI`, `LAKSHMI_PUJA` | Telangana's rule | Telangana's list (C1528) |
| Raksha Bandhan, Bhaiyadooj (Chitragupta Jayanti) | `RAKSHABANDHAN`, `YAMA_DWITIYA` | `NEPAL`'s rule | Nepal's pack (C1528) |
| Dussehra (Mahanavami) | `MAHA_NAVAMI` | `NEPAL_PANCHANGA`'s rule | a printed panchanga (C1528) |
| Holika Dahan | `HOLIKA` | `DHARMASINDHU`'s rule | *Dharmasindhu* (C1524) |
| Narak Chaturdashi (restricted) | `NARAK_CHATURDASHI` | the day before `LAKSHMI_PUJA` | the notifications (C1528) |
| Govardhan Puja; Maharaja Agrasen Jayanti (restricted) | `GOVARDHAN_PUJA`, `AGRASEN_JAYANTI` | *Dharmasindhu*'s Bali pratipada and Navaratra arambha | *Dharmasindhu* p. 78 (C1523) |
| Basant Panchami, Chetichand, Parshuram Jayanti, Anant Chaturdashi, Mahashtami, Chhath (restricted) | `BASANTA_PANCHAMI`, `CHETI_CHAND`, `PARASHURAMA_JAYANTI`, `ANANTA_CHATURDASHI`, `MAHA_ASHTAMI`, `CHHATH` | `NEPAL_PANCHANGA`'s rule; Chetichand its Samvatsara arambha, as Gujarat's list keeps it | a printed panchanga |
| Maharshi Valmiki Jayanti (restricted) | `VALMIKI_JAYANTI` | Karnataka's rule, Ashvina's full moon at sunrise | the notifications; Karnataka's |
| Sant Ravidas Jayanti (restricted) | `RAVIDAS_JAYANTI` | Magha's full moon at sunrise | the notifications (C1521) |
| Holi, second day (restricted) | `HOLI_SECOND_DAY` | the second day after `HOLIKA` | the notification |
| Makar Sankranti (restricted) | `UTTARAYANA_PUNYAKALA` | the first of Makara of `TAMIL_SOLAR`: the ingress's day when it falls before sunset, else the next | the notifications (C1522) |
| Easter Saturday, Easter Monday (restricted) | `EASTER_SATURDAY`, `EASTER_MONDAY` | the day before and after Easter, Gregorian computus | the notification |
| Dr Bhimrao Ambedkar Jayanti; New Year's Day, Jannayak Karpoori Thakur Jayanti, Maharshi Kashyap and Maharaja Nishadraj Guhya Jayanti, Chandra Shekhar Jayanti, Maharana Pratap Jayanti, Vishwakarma Puja, Sardar Vallabhbhai Patel and Acharya Narendra Dev Jayanti, Veerangana Uda Devi's martyrdom day, Chaudhary Charan Singh Jayanti, Christmas Eve (restricted) | `AMBEDKAR_JAYANTI`, `NEW_YEAR_S_DAY_AD`, `KARPOORI_THAKUR_JAYANTI`, `KASHYAP_NISHADRAJ_JAYANTI`, `CHANDRASHEKHAR_JAYANTI`, `MAHARANA_PRATAP_JAYANTI`, `VISHWAKARMA_PUJA`, `SARDAR_PATEL_JAYANTI`, `UDA_DEVI_SHAHEED_DIWAS`, `CHARAN_SINGH_JAYANTI`, `CHRISTMAS_EVE` | a Gregorian date | the notification |

The notification's tithis are purnimanta: "भाद्रपद कृष्ण 08" is the amanta
Shravana's dark 8th, "कार्तिक कृष्ण 15" Ashvina's new moon. The rules name the
amanta month, as every shipped rule does, and the calendar crate turns a dark
fortnight's month by one for a purnimanta reading; no rule depends on it. One new
lunar rule was needed, Ravidas Jayanti, and no new form. The new keys are
`RAVIDAS_JAYANTI`, `AGRASEN_JAYANTI`, `HOLI_SECOND_DAY`, `EASTER_MONDAY` and six
civil days.

## 3. The sky

The lunar days are reckoned under the modern sky, the built-in ephemeris with
the default profile. Makar Sankranti uses `TAMIL_SOLAR`'s fixed place, Chennai,
on India's clock: its first of Makara is the day of the ingress when the Sun
enters before sunset, and the next day otherwise (C1522).

## 4. The days printed, not reckoned

`PublishedSet::india_uttar_pradesh` (`in-up-gad-printed`) carries the twelve
Islamic days the asterisk marks, as printed, each marked `subjectTo:
MOON_SIGHTING`:

- **Public**: Hazrat Ali's birthday, Id-ul-Fitr, Id-uz-Zuha (Bakrid), Muharram
  and Barawafat (Eid-e-Milad), as `HAZRAT_ALI_BIRTHDAY`, `EID_AL_FITR`,
  `EID_AL_ADHA`, `MUHARRAM` and `MAWLID`.
- **Restricted**, `optional`: the Urs of Khwaja Moinuddin Chishti, Shab-e-Barat,
  Jamat-ul-Vida, the restricted days beside Id-ul-Fitr, Bakrid and Muharram, and
  Chehlum, as `KHWAJA_GARIB_NAWAZ_URS`, `SHAB_E_BARAT`, `JUMAT_UL_VIDA`,
  `EID_AL_FITR_NEXT_DAY`, `EID_AL_ADHA_NEXT_DAY`, `MUHARRAM_SECOND_DAY` and
  `ARBAEEN`.

Each stands in all eight years: 96 days.

## 5. Against the print

At Lucknow, 315 of the 328 rows meet the print (`festival-measured.md` §20,
and per year in `crates/sdk/tests/uttar_pradesh.rs`). The rows are each year's
reckoned days of both lists and the fifteen fixed days, which stand in every
year:

| year | met / printed |
|---|---|
| 2019 | 39 / 41 |
| 2020 | 39 / 41 |
| 2021 | 41 / 41 |
| 2022 | 40 / 41 |
| 2023 | 40 / 41 |
| 2024 | 41 / 41 |
| 2025 | 38 / 41 |
| 2026 | 37 / 41 |

Each lunar day's rule is the shipped rule that meets the most notifications.
Every shipped pack's rule for each key was measured at Lucknow against the eight
years (C1528). *Dharmasindhu*'s Raksha Bandhan, Janmashtami, Mahanavami and
Bhaiyadooj, the first choice, met 6, 5, 5 and 7 of 8. Nepal's Raksha Bandhan
and Bhaiyadooj meet all eight; the central list's Janmashtami meets six, the
print's Mahanavami seven. Telangana's Dussehra and Deepawali meet seven and eight
where the central list's meet six and seven. No shipped rule for Narak
Chaturdashi meets more than four years, so the list's own reading is kept: it
falls the day before Deepawali.

Thirteen rows part, each named with its cause. No rule was fitted to them:

- **The day the tithi begins or fills, not the sunrise's** (C1529). In each of
  these the notification lists the earlier day, which the tithi fills or begins
  on, where the rule takes the day whose sunrise it holds:
  - Janmashtami in 2019 and 2022, on the day whose midnight holds the dark 8th.
  - Anant Chaturdashi and Mahanavami in 2020 (C1525).
  - Govardhan Puja in 2023 and 2026, on the day Kartika's bright 1st begins.
  - Basant Panchami in 2025, whose 5th held the 3rd's sunrise by two minutes.
- **The later day**:
  - **Dussehra, 2019.** The 10th held both afternoons, and Telangana's rule
    keeps the earlier.
  - **Chhath, 2025.** The 6th held both sunrises, and the print's rule keeps
    the earlier.
- **Holi and its second day, 2026** (C854, C1445). The notification lists both
  a day later than the fire's count, as the central list's notice does.
- **Narak Chaturdashi, 2025 and 2026** (C1528). The notification lists it on
  Deepawali's own day, whose sunrise the dark 14th holds.

## 6. What is not here

- **Para 4, para 5 and para 6** (C1527): the banks' closing of accounts on 1
  April, the days declared by executive order, and a district's local holidays.
- **Later orders that move or add a day** (C1526).
- **A rule for Narak Chaturdashi's two kinds of year.** No shipped rule meets
  both: the day before Deepawali in six years, and Deepawali's own day in two
  (C1528).
