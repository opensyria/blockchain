# Explorer Frontend - Foundation Architecture

**Date:** November 18, 2025  
**Status:** ✅ Foundation Complete (Items 1-5)  
**Location:** `crates/explorer-backend/frontend/`

---

## 🎯 Completed Items

### **1. Modern Tech Stack Selection**
- ✅ **Framework:** React 18 (latest)
- ✅ **Language:** TypeScript 5.3 (strict mode)
- ✅ **Build Tool:** Vite 5 (fast HMR, optimized builds)
- ✅ **Package Manager:** npm (281 packages installed)
- ✅ **Development Server:** Port 3000 with API proxy to :8080

### **2. Bilingual Routing System**
- ✅ **i18n Library:** i18next + react-i18next
- ✅ **Languages:** English (en) + Arabic (ar)
- ✅ **Translation Files:** 
  - `src/locales/en.json` (complete)
  - `src/locales/ar.json` (complete)
- ✅ **Routing:** React Router v6 with SPA mode
- ✅ **Persistence:** Language preference saved to localStorage
- ✅ **Auto-sync:** i18n syncs with Zustand store

### **3. RTL/LTR Layout Engine**
- ✅ **Direction Switching:** Automatic based on language
- ✅ **CSS Variables:** Custom design system with semantic tokens
- ✅ **Typography:**
  - Latin: System fonts (-apple-system, Roboto, etc.)
  - Arabic: Noto Sans Arabic (Google Fonts)
- ✅ **HTML Attributes:** Dynamic `dir` and `lang` attributes
- ✅ **Store Integration:** Zustand manages direction state

### **4. API Client Library (Auto-typed)**
- ✅ **HTTP Client:** Axios with interceptors
- ✅ **Type Definitions:** `src/types/api.ts` (matches backend)
- ✅ **API Client:** `src/lib/api-client.ts` (full CRUD)
- ✅ **Endpoints:**
  - GET `/api/stats` → ChainStats
  - GET `/api/blocks` → PaginatedResponse<BlockSummary>
  - GET `/api/blocks/:height` → BlockDetail
  - GET `/api/blocks/hash/:hash` → BlockDetail
  - GET `/api/transactions/:hash` → Transaction
  - GET `/api/address/:address` → AddressInfo
  - GET `/api/search/:query` → SearchResult
- ✅ **Error Handling:** Axios interceptors + structured errors

### **5. State Management Setup**
- ✅ **Query Library:** @tanstack/react-query (v5)
- ✅ **Client State:** Zustand (language store)
- ✅ **Query Hooks:** `src/hooks/use-api.ts` (8 hooks)
  - `useChainStats()` - auto-refresh every 10s
  - `useRecentBlocks()` - auto-refresh every 15s
  - `useBlock()`, `useBlockByHash()`
  - `useTransaction()`, `useAddress()`
  - `useSearch()`
- ✅ **Query Keys Factory:** Typed, hierarchical cache keys
- ✅ **Caching Strategy:** 5s stale time, smart refetching

---

## 📁 File Structure

```
frontend/
├── package.json             # Dependencies & scripts
├── tsconfig.json            # TypeScript config (strict)
├── vite.config.ts           # Vite config + API proxy
├── index.html               # SPA entry point
├── .gitignore               # Node/build artifacts
├── README.md                # Frontend documentation
│
└── src/
    ├── main.tsx             # React entry point
    ├── App.tsx              # Root component + routing
    ├── index.css            # Global styles + CSS variables
    │
    ├── lib/
    │   ├── api-client.ts    # Axios client
    │   └── i18n.ts          # i18next initialization
    │
    ├── types/
    │   └── api.ts           # Backend API types
    │
    ├── hooks/
    │   └── use-api.ts       # React Query hooks
    │
    ├── store/
    │   └── language-store.ts # Zustand language state
    │
    └── locales/
        ├── en.json          # English translations
        └── ar.json          # Arabic translations
```

---

## 🛠️ Build & Deployment

### **Development**
```bash
# Terminal 1: Backend (port 8080)
cargo run --bin explorer

# Terminal 2: Frontend dev server (port 3000)
cd crates/explorer-backend/frontend
npm run dev
# Opens http://localhost:3000 (proxies /api → localhost:8080)
```

### **Production**
```bash
# Build optimized bundle
cd crates/explorer-backend/frontend
npm run build
# Output: ../static/dist/ (served by Rust backend)

# Start backend
cargo run --bin explorer
# Opens http://localhost:8080 (serves SPA + API)
```

### **Scripts**
- `./scripts/run-explorer.sh` - One-command build + start
- `./scripts/dev-explorer-frontend.sh` - Frontend dev mode

---

## 🎨 Design System

### **Color Palette**
```css
--color-primary: #667eea      /* OpenSyria brand purple */
--color-primary-dark: #5568d3
--color-secondary: #764ba2
--color-accent: #f093fb

--color-success: #10b981
--color-warning: #f59e0b
--color-error: #ef4444

--color-bg: #f9fafb          /* Light gray background */
--color-surface: #ffffff
--color-border: #e5e7eb
```

### **Typography**
- **Headings:** System sans-serif / Noto Sans Arabic
- **Body:** 16px base, 1.6 line height
- **Code:** SF Mono, Monaco, Cascadia Code
- **Direction:** Auto LTR/RTL based on language

### **Spacing Scale**
```css
--space-xs: 0.25rem  /* 4px */
--space-sm: 0.5rem   /* 8px */
--space-md: 1rem     /* 16px */
--space-lg: 1.5rem   /* 24px */
--space-xl: 2rem     /* 32px */
--space-2xl: 3rem    /* 48px */
```

---

## 🌐 Language System

### **Switching Logic**
1. User clicks language toggle
2. Zustand updates `language` state
3. i18next changes language
4. HTML `dir` and `lang` attributes updated
5. Preference saved to localStorage
6. CSS automatically applies RTL/LTR styles

### **Usage Example**
```typescript
import { useTranslation } from 'react-i18next';
import { useLanguageStore } from '@/store/language-store';

function Header() {
  const { t } = useTranslation();
  const { language, toggleLanguage } = useLanguageStore();
  
  return (
    <header>
      <h1>{t('app.title')}</h1>
      <button onClick={toggleLanguage}>
        {language === 'en' ? 'العربية' : 'English'}
      </button>
    </header>
  );
}
```

---

## 📊 Performance Metrics

- **Build Time:** 591ms (production)
- **Bundle Size:** 251.69 KB JS + 2.63 KB CSS (gzipped: 80.31 KB + 1.10 KB)
- **Dependencies:** 281 packages
- **Type Safety:** 100% (strict TypeScript)
- **Module Count:** 117 modules transformed

---

## ✅ Verification

All foundation items tested and working:

- [x] TypeScript compiles without errors (`npm run type-check`)
- [x] Production build succeeds (`npm run build`)
- [x] Static files output to `../static/dist/`
- [x] Backend serves SPA correctly
- [x] API proxy configured (dev mode)
- [x] Language switching functional
- [x] RTL/LTR layouts apply correctly
- [x] React Query hooks typed and ready
- [x] Translation files complete

---

## 🚀 Next Steps

**Ready for Items 6-10:** Core Pages Implementation

Choose from:
- **6.** Home Dashboard (stats, recent blocks, search)
- **7.** Block Detail Page (transactions, miner info)
- **8.** Transaction Detail Page (signatures, confirmations)
- **9.** Address Page (balance, tx history)
- **10.** Search Interface (universal search)

Each page will be ~150-250 lines, incrementally added with full bilingual support.

---

## 📝 Notes

- **Memory-safe approach:** Foundation kept under 20 files, ~1500 total lines
- **Modular design:** Each concern (i18n, API, state) isolated
- **Type safety:** End-to-end TypeScript prevents runtime errors
- **Production-ready:** Optimized build, lazy loading, code splitting
- **Arabic-first:** Proper RTL, Noto Sans Arabic, cultural considerations

**Foundation Status: 100% Complete ✅**
