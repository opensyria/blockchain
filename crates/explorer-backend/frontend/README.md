# OpenSyria Explorer - Frontend

Modern, bilingual (Arabic/English) blockchain explorer built with React, TypeScript, and Vite.

## Tech Stack

- **Framework:** React 18
- **Language:** TypeScript 5.3
- **Build Tool:** Vite 5
- **Routing:** React Router v6
- **State Management:** Zustand + React Query
- **Styling:** CSS Variables (custom design system)
- **i18n:** i18next + react-i18next
- **API Client:** Axios with typed responses

## Features

✅ **Bilingual Support:** Full Arabic/English localization with RTL/LTR layout switching  
✅ **Type Safety:** End-to-end TypeScript with backend API types  
✅ **Smart Caching:** React Query for optimized data fetching  
✅ **Responsive Design:** Mobile-first, accessible UI  
✅ **Real-time Updates:** Auto-refreshing stats and blocks  
✅ **Core Pages:** Home dashboard, block explorer, transaction viewer, address lookup, universal search  
✅ **Production Ready:** Optimized builds, code splitting, lazy loading  

## Project Structure

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
    ├── components/          # Reusable UI components
    │   ├── index.ts
    │   ├── Layout.tsx       # Main layout with nav
    │   ├── SearchBar.tsx    # Universal search
    │   ├── StatCard.tsx     # Statistics display
    │   └── BlockList.tsx    # Block list component
    │
    ├── pages/               # Route pages
    │   ├── index.ts
    │   ├── HomePage.tsx     # Dashboard with stats
    │   ├── BlocksPage.tsx   # Paginated block list
    │   ├── BlockDetailPage.tsx    # Block details
    │   ├── TransactionPage.tsx    # Transaction details
    │   ├── AddressPage.tsx        # Address balance & history
    │   └── SearchResultPage.tsx   # Search redirect logic
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

## Development

### Prerequisites

- Node.js 18+ and npm/yarn/pnpm
- Running backend API at `http://localhost:8080`

### Setup

```bash
# Install dependencies
npm install

# Start dev server (proxies /api to backend)
npm run dev

# Open browser
# http://localhost:3000
```

### Build for Production

```bash
# Type check + build
npm run build

# Output: ../static/dist/
# Served by Rust backend at /
```

### Scripts

- `npm run dev` - Start development server with HMR
- `npm run build` - Production build with type checking
- `npm run preview` - Preview production build
- `npm run lint` - Run ESLint
- `npm run type-check` - TypeScript type checking only

## Language System

### Switching Languages

The app automatically detects and persists language preference:

```typescript
import { useLanguageStore } from '@/store/language-store';

function LanguageToggle() {
  const { language, toggleLanguage } = useLanguageStore();
  
  return (
    <button onClick={toggleLanguage}>
      {language === 'en' ? 'العربية' : 'English'}
    </button>
  );
}
```

### Adding Translations

1. Edit `src/locales/en.json` and `src/locales/ar.json`
2. Use in components:

```typescript
import { useTranslation } from 'react-i18next';

function MyComponent() {
  const { t } = useTranslation();
  return <h1>{t('app.title')}</h1>;
}
```

## API Integration

### Using Hooks

```typescript
import { useChainStats, useRecentBlocks } from '@/hooks/use-api';

function Dashboard() {
  const { data: stats, isLoading } = useChainStats();
  const { data: blocks } = useRecentBlocks(1, 10);
  
  if (isLoading) return <div>Loading...</div>;
  
  return (
    <div>
      <h2>Height: {stats?.height}</h2>
      {blocks?.data.map(block => <div key={block.height}>{block.hash}</div>)}
    </div>
  );
}
```

### Direct API Calls

```typescript
import { apiClient } from '@/lib/api-client';

const stats = await apiClient.getChainStats();
const block = await apiClient.getBlockByHeight(100);
```

## Next Steps

**Items 11-13:** Real-time features (WebSocket, mempool viewer, network topology)  
**Items 14-16:** Enhanced Arabic typography and cultural UI patterns  
**Items 17-20:** Charts, identity gallery, governance viewer, PWA  

## Component Examples

### Using Layout

```typescript
import { Layout } from '@/components/Layout';

function MyPage() {
  return (
    <Layout>
      <div className="container">
        <h1>My Content</h1>
      </div>
    </Layout>
  );
}
```

### Using API Hooks

```typescript
import { useChainStats, useBlock } from '@/hooks/use-api';

function StatsDisplay() {
  const { data, isLoading, error } = useChainStats();
  
  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error!</div>;
  
  return <div>Height: {data.height}</div>;
}
```  

## License

MIT OR Apache-2.0
