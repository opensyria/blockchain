# OpenSyria Brand Guidelines

**Version:** 1.0  
**Last Updated:** November 18, 2025  
**Status:** Official Brand Standard

---

## Official Brand Identity

### Primary Names

**Full Official Name:** OpenSyria: Digital Lira Blockchain  
**Platform Name:** OpenSyria  
**Currency Name:** Syrian Digital Lira  
**Currency Code:** SYL  
**Tagline:** "For ALL Syrians, wherever they are"

### Arabic Names

**Full Name:** أوبن سيريا: بلوكتشين الليرة الرقمية السورية  
**Platform:** أوبن سيريا (OpenSyria)  
**Currency:** الليرة الرقمية السورية  
**Currency Code:** ل.س.ر  

---

## Brand Architecture

```
OpenSyria (Platform)
├── Syrian Digital Lira (Primary Product - Currency)
├── Heritage NFTs (Secondary Product)
└── Governance (Secondary Product)
```

**Hierarchy:**
1. **OpenSyria** = Umbrella brand (blockchain platform)
2. **Syrian Digital Lira** = Primary product (currency/token)
3. **Heritage NFTs** = Cultural feature
4. **Governance** = Community feature

---

## Usage Rules

### First Mention Rule

**In Documents:**
Always use full name on first mention:
- ✅ "OpenSyria: Digital Lira blockchain"
- ✅ "أوبن سيريا: بلوكتشين الليرة الرقمية السورية"

**Subsequent Mentions:**
Use context-appropriate short name:
- Platform context: "OpenSyria"
- Currency context: "Syrian Digital Lira" or "SYL"

### Platform Context

When discussing blockchain features, infrastructure, or technology:
- ✅ "OpenSyria blockchain"
- ✅ "OpenSyria governance system"
- ✅ "Built on OpenSyria"
- ✅ "OpenSyria consensus mechanism"
- ✅ "Deploy an OpenSyria node"

**Arabic:**
- ✅ "بلوكتشين أوبن سيريا"
- ✅ "نظام حوكمة أوبن سيريا"
- ✅ "مبني على أوبن سيريا"

### Currency Context

When discussing payments, wallets, or economic features:
- ✅ "Send Syrian Digital Lira"
- ✅ "1,000 SYL"
- ✅ "SYL wallet"
- ✅ "Pay in SYL"
- ✅ "Digital Lira balance"

**Arabic:**
- ✅ "إرسال الليرة الرقمية السورية"
- ✅ "١٬٠٠٠ ل.س.ر"
- ✅ "محفظة ل.س.ر"

### Code/Technical Context

**Crate Names:** (Keep existing - one word, lowercase)
- `opensyria_core`
- `opensyria_consensus`
- `opensyria_wallet`
- etc.

**Import Statements:**
```rust
use opensyria_core::Block;
use opensyria_wallet::Wallet;
```

**Clarification in README:**
"OpenSyria (one word) in code, two words in documentation"

---

## Capitalization Standards

### User-Facing Text (UI, Docs, Marketing)

**English:**
- "OpenSyria" (one word, capital O and S)
- "Syrian Digital Lira" (title case)
- "Digital Lira" (when Syrian is implied)

**Arabic:**
- "أوبن سيريا" (Latin letters in Arabic text)
- "الليرة الرقمية السورية" (standard Arabic)

### URLs & Domains

- opensyria.io (lowercase, no space)
- opensyria.org (lowercase, no space)
- github.com/opensyria/blockchain

### Social Media

**Handles:**
- Twitter/X: @OpenSyria
- GitHub: @opensyria
- Discord: OpenSyria
- Telegram: @OpenSyria

**Hashtags:**
- #OpenSyria (capital O and S, no space)
- #SyrianDigitalLira
- #SYL

---

## Currency Specifications

### Official Currency Name

**Full Name:** Syrian Digital Lira  
**Short Name:** Digital Lira (when context is clear)  
**Currency Code:** SYL  
**Arabic Full:** الليرة الرقمية السورية  
**Arabic Code:** ل.س.ر

### Currency Code Breakdown

**SYL** stands for:
- **S**yrian
- (Digital) **L**ira

**ل.س.ر** stands for:
- **ل**يرة (**L**ira)
- **س**ورية (**S**yrian)
- **ر**قمية (Digi**t**al)

### Display Formats

**Amounts:**
```
English: 1,234.56 SYL
Arabic:  ١٬٢٣٤٫٥٦ ل.س.ر

Small: 0.001 SYL
Large: 1,000,000 SYL (or 1M SYL)
```

**In Sentences:**
- "Send 100 SYL to..."
- "Balance: 5,432.10 SYL"
- "Fee: 0.01 SYL"

---

## Application Titles

### Frontend Applications

**Block Explorer:**
- English: "OpenSyria Explorer"
- Subtitle: "Syrian Digital Lira (SYL) Blockchain"
- Arabic: "مستكشف أوبن سيريا"
- Subtitle AR: "بلوكتشين الليرة الرقمية السورية"

**Page Titles (HTML):**
```html
<title>OpenSyria Explorer - Syrian Digital Lira Blockchain</title>
```

### CLI Tools

**Wallet CLI:**
```rust
#[command(about = "Syrian Digital Lira Wallet (OpenSyria) | محفظة الليرة الرقمية السورية (أوبن سيريا)")]
```

**Node CLI:**
```rust
#[command(about = "OpenSyria Blockchain Node | عقدة بلوكتشين أوبن سيريا")]
```

**Miner CLI:**
```rust
#[command(about = "OpenSyria Miner | منقّب أوبن سيريا")]
```

**Governance CLI:**
```rust
#[command(about = "OpenSyria Governance | حوكمة أوبن سيريا")]
```

---

## Translation Parity Requirements

### Parallel Structure Rule

Arabic and English MUST have parallel structure:

**✅ CORRECT:**
```
EN: "OpenSyria Blockchain"
AR: "بلوكتشين أوبن سيريا"
Both mention platform name ✅
```

**❌ INCORRECT:**
```
EN: "OpenSyria Digital Lira Wallet"
AR: "محفظة الليرة الرقمية السورية"
English has OpenSyria, Arabic doesn't ❌
```

### Required Elements

Both languages must include:
1. ✅ Platform name (OpenSyria / أوبن سيريا)
2. ✅ Product name (if applicable)
3. ✅ Same level of detail
4. ✅ Same formality level

### Validation Checklist

For ALL user-facing text:
- [ ] English mentions "OpenSyria" → Arabic mentions "أوبن سيريا"
- [ ] English mentions "Syrian Digital Lira" → Arabic mentions "الليرة الرقمية السورية"
- [ ] Word count approximately matches (±20%)
- [ ] Concepts are identical (not just similar)
- [ ] Formality matches (both formal or both casual)

---

## Cultural Sensitivity Guidelines

### Required Disclaimers

**All official materials MUST include:**

> **OpenSyria is an independent, non-governmental project.**  
> Not affiliated with any Syrian government entity. Open to ALL Syrians regardless of location or political affiliation.

**Arabic:**
> **أوبن سيريا مشروع مستقل وغير حكومي.**  
> غير مرتبط بأي كيان حكومي سوري. مفتوح لجميع السوريين بغض النظر عن الموقع أو الانتماء السياسي.

### Inclusive Language

**Always emphasize:**
- ✅ "For ALL Syrians"
- ✅ "Inside Syria and diaspora"
- ✅ "Wherever they are"
- ✅ "Borderless community"
- ✅ "No geographic restrictions"

**Never imply:**
- ❌ Government affiliation
- ❌ Political alignment
- ❌ Geographic exclusivity
- ❌ Nationality requirements

### Diaspora Inclusion

**Explicitly mention:**
- Syrian diaspora (6.6M worldwide)
- Refugees welcome
- No border restrictions
- Global accessibility

---

## Visual Identity (Future)

### Color Palette (Proposed)

**Option 1: Neutral Tech Colors (RECOMMENDED)**
- Primary: #3B82F6 (Blue)
- Secondary: #10B981 (Green)  
- Accent: #F59E0B (Amber)
- Background: #F9FAFB (Light Gray)
- Text: #111827 (Dark Gray)

**Option 2: Syrian Heritage Colors**
- Red: #CE1126
- White: #FFFFFF
- Black: #000000
- Green: #007A3D

**Note:** Color palette to be finalized by design team.

### Typography

**English:**
- Headings: Inter (Bold/SemiBold)
- Body: Inter (Regular)
- Code: JetBrains Mono

**Arabic:**
- Headings: Noto Sans Arabic (Bold)
- Body: Noto Sans Arabic (Regular)  
- Code: JetBrains Mono (stays LTR)

### Logo (Future)

- Logo design TBD
- Must work in both LTR and RTL layouts
- Must be readable at small sizes (16x16px favicon)
- Should incorporate Syrian cultural elements subtly

---

## Marketing Guidelines

### Elevator Pitch

**English (30 seconds):**
> "OpenSyria is a blockchain platform powering the Syrian Digital Lira (SYL) - a decentralized cryptocurrency for ALL Syrians, whether in Damascus or diaspora. Built on Rust, it combines secure digital payments with Syrian cultural heritage preservation through NFTs and community governance."

**Arabic:**
> "أوبن سيريا منصة بلوكتشين تشغّل الليرة الرقمية السورية - عملة مشفرة لامركزية لجميع السوريين، في دمشق أو المهجر. مبنية على Rust، تجمع بين المدفوعات الرقمية الآمنة والحفاظ على التراث الثقافي السوري من خلال NFTs والحوكمة المجتمعية."

### Key Messages

1. **Independent & Decentralized**
   - Not government-controlled
   - Permissionless
   - Open-source

2. **Culturally Authentic**
   - Bilingual (Arabic/English)
   - Heritage NFTs
   - Syrian identity at core

3. **Inclusive & Borderless**
   - For ALL Syrians
   - Diaspora welcome
   - No restrictions

4. **Technically Sound**
   - Built on Rust
   - Proof-of-Work consensus
   - Professional engineering

### Forbidden Claims

**Never claim:**
- ❌ "Official Syrian currency"
- ❌ "Government-backed"
- ❌ "Legal tender"
- ❌ "Replacement for Syrian Pound"
- ❌ "Endorsed by any government"

**Instead say:**
- ✅ "Independent digital currency"
- ✅ "Community-driven"
- ✅ "Decentralized alternative"
- ✅ "Complementary to existing systems"

---

## SEO & Discoverability

### Primary Keywords

**English:**
- OpenSyria blockchain
- Syrian Digital Lira
- SYL cryptocurrency
- Syrian blockchain
- Digital Lira

**Arabic:**
- أوبن سيريا
- الليرة الرقمية السورية
- ل.س.ر
- بلوكتشين سوري

### Meta Descriptions

**Homepage:**
```html
<meta name="description" content="OpenSyria: Syrian Digital Lira (SYL) blockchain. Decentralized cryptocurrency and cultural heritage platform for ALL Syrians. Independent, open-source, bilingual.">
```

**Arabic:**
```html
<meta name="description" content="أوبن سيريا: بلوكتشين الليرة الرقمية السورية. عملة مشفرة لامركزية ومنصة تراث ثقافي لجميع السوريين. مستقل، مفتوح المصدر، ثنائي اللغة.">
```

---

## Legal & Trademark

### Trademark Status

**Current:** Unregistered  
**Planned:** Community trademark registration  
**Model:** Similar to Linux Foundation

### Domain Ownership

**Registered:**
- opensyria/blockchain (GitHub)

**To Register:**
- [ ] opensyria.io
- [ ] opensyria.org  
- [ ] syriandl.com (redirect)
- [ ] digitallira.sy (if available)

### Licensing

**Code:** MIT License + Apache 2.0 (dual license)  
**Documentation:** CC BY-SA 4.0  
**Brand Assets:** TBD (community trademark)

---

## Compliance Checklist

Before ANY public-facing material is published:

- [ ] Uses "OpenSyria: Digital Lira" on first mention
- [ ] Currency code is "SYL" (not "SDL", "DSYL", or other)
- [ ] Arabic translation is parallel to English
- [ ] Includes independence disclaimer
- [ ] Mentions diaspora inclusivity
- [ ] No government affiliation implied
- [ ] Color palette matches brand (when applicable)
- [ ] Typography follows guidelines
- [ ] SEO keywords included
- [ ] Legal disclaimers present

---

## Version History

**v1.0 (November 18, 2025):**
- Initial brand guidelines
- Dual branding strategy adopted
- OpenSyria (platform) + Syrian Digital Lira (currency)
- Currency code: SYL
- Translation parity requirements
- Cultural sensitivity guidelines

---

## Contact

**Brand Questions:** Create issue in opensyria/blockchain  
**Design Assets:** TBD  
**Press Inquiries:** TBD

---

**This is a living document. Last updated: November 18, 2025**
