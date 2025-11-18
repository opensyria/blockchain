import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Layout } from '@/components/Layout';
import './IdentityPage.css';

interface HeritageToken {
  id: string;
  name: string;
  nameAr: string;
  type: 'site' | 'craft' | 'cuisine' | 'music' | 'art' | 'manuscript';
  category: 'ancient' | 'islamic' | 'ottoman' | 'modern' | 'contemporary';
  city: string;
  imageUrl: string;
  description: string;
  ipfsCid?: string;
}

// Mock data - would come from API
const heritageTokens: HeritageToken[] = [
  {
    id: 'damascus-umayyad-mosque',
    name: 'Umayyad Mosque',
    nameAr: 'الجامع الأموي',
    type: 'site',
    category: 'islamic',
    city: 'Damascus',
    imageUrl: 'https://images.unsplash.com/photo-1578070181910-f1e514afdd08?w=400',
    description: 'One of the largest and oldest mosques in the world, built in 715 AD',
    ipfsCid: 'QmXyz123...',
  },
  {
    id: 'aleppo-citadel',
    name: 'Aleppo Citadel',
    nameAr: 'قلعة حلب',
    type: 'site',
    category: 'ancient',
    city: 'Aleppo',
    imageUrl: 'https://images.unsplash.com/photo-1591608971362-f08b2a75731a?w=400',
    description: 'Medieval fortified palace dating back to 3000 BC',
    ipfsCid: 'QmAbc456...',
  },
  {
    id: 'damascus-steel',
    name: 'Damascus Steel Craftsmanship',
    nameAr: 'حرفة الفولاذ الدمشقي',
    type: 'craft',
    category: 'islamic',
    city: 'Damascus',
    imageUrl: 'https://images.unsplash.com/photo-1595246140625-573b715d11dc?w=400',
    description: 'Traditional Damascus steel forging techniques',
  },
  {
    id: 'palmyra-ruins',
    name: 'Palmyra Ancient Ruins',
    nameAr: 'آثار تدمر',
    type: 'site',
    category: 'ancient',
    city: 'Palmyra',
    imageUrl: 'https://images.unsplash.com/photo-1541888946425-d81bb19240f5?w=400',
    description: 'UNESCO World Heritage Site with Greco-Roman architecture',
    ipfsCid: 'QmDef789...',
  },
  {
    id: 'damascene-brocade',
    name: 'Damascene Brocade',
    nameAr: 'الديباج الدمشقي',
    type: 'craft',
    category: 'islamic',
    city: 'Damascus',
    imageUrl: 'https://images.unsplash.com/photo-1610701596007-11502861dcfa?w=400',
    description: 'Traditional silk weaving with gold and silver threads',
  },
  {
    id: 'syrian-cuisine',
    name: 'Syrian Culinary Heritage',
    nameAr: 'التراث الطهوي السوري',
    type: 'cuisine',
    category: 'contemporary',
    city: 'Damascus',
    imageUrl: 'https://images.unsplash.com/photo-1599487488170-d11ec9c172f0?w=400',
    description: 'Ancient recipes and cooking traditions',
  },
];

export function IdentityPage() {
  const { i18n } = useTranslation();
  const [selectedType, setSelectedType] = useState<string>('all');
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [selectedToken, setSelectedToken] = useState<HeritageToken | null>(null);

  const filteredTokens = heritageTokens.filter(token => {
    if (selectedType !== 'all' && token.type !== selectedType) return false;
    if (selectedCategory !== 'all' && token.category !== selectedCategory) return false;
    return true;
  });

  return (
    <Layout>
      <div className="container">
        <div className="identity-page animate-fade-in-up">
          <div className="page-header corner-ornament">
            <div>
              <h1 className="page-title">Syrian Cultural Heritage</h1>
              <p className="page-subtitle">Preserving Syria's identity on the blockchain</p>
            </div>
          </div>

          <div className="filters-section card-cultural">
            <div className="filter-group">
              <label className="filter-label">Type:</label>
              <div className="filter-buttons">
                <button
                  className={`filter-btn ${selectedType === 'all' ? 'active' : ''}`}
                  onClick={() => setSelectedType('all')}
                >
                  All
                </button>
                <button
                  className={`filter-btn ${selectedType === 'site' ? 'active' : ''}`}
                  onClick={() => setSelectedType('site')}
                >
                  🏛️ Sites
                </button>
                <button
                  className={`filter-btn ${selectedType === 'craft' ? 'active' : ''}`}
                  onClick={() => setSelectedType('craft')}
                >
                  ✋ Crafts
                </button>
                <button
                  className={`filter-btn ${selectedType === 'cuisine' ? 'active' : ''}`}
                  onClick={() => setSelectedType('cuisine')}
                >
                  🍽️ Cuisine
                </button>
              </div>
            </div>

            <div className="filter-group">
              <label className="filter-label">Period:</label>
              <div className="filter-buttons">
                <button
                  className={`filter-btn ${selectedCategory === 'all' ? 'active' : ''}`}
                  onClick={() => setSelectedCategory('all')}
                >
                  All
                </button>
                <button
                  className={`filter-btn ${selectedCategory === 'ancient' ? 'active' : ''}`}
                  onClick={() => setSelectedCategory('ancient')}
                >
                  Ancient
                </button>
                <button
                  className={`filter-btn ${selectedCategory === 'islamic' ? 'active' : ''}`}
                  onClick={() => setSelectedCategory('islamic')}
                >
                  Islamic
                </button>
                <button
                  className={`filter-btn ${selectedCategory === 'contemporary' ? 'active' : ''}`}
                  onClick={() => setSelectedCategory('contemporary')}
                >
                  Contemporary
                </button>
              </div>
            </div>
          </div>

          <div className="tokens-grid stagger-children">
            {filteredTokens.map((token) => (
              <div
                key={token.id}
                className="token-card card-cultural hover-lift"
                onClick={() => setSelectedToken(token)}
              >
                <div className="token-image" style={{ backgroundImage: `url(${token.imageUrl})` }}>
                  {token.ipfsCid && (
                    <div className="ipfs-badge badge-heritage">
                      <span>📦 IPFS</span>
                    </div>
                  )}
                </div>
                <div className="token-content">
                  <h3 className="token-title">
                    {i18n.language === 'ar' ? token.nameAr : token.name}
                  </h3>
                  <div className="token-meta">
                    <span className="token-city">📍 {token.city}</span>
                    <span className="token-category">{token.category}</span>
                  </div>
                  <p className="token-description">{token.description}</p>
                </div>
              </div>
            ))}
          </div>

          {filteredTokens.length === 0 && (
            <div className="empty-state">
              <p>No heritage tokens found matching your filters</p>
            </div>
          )}

          {selectedToken && (
            <div className="modal-overlay" onClick={() => setSelectedToken(null)}>
              <div className="modal-content card-cultural animate-scale-in" onClick={(e) => e.stopPropagation()}>
                <button className="modal-close" onClick={() => setSelectedToken(null)}>
                  ✕
                </button>
                <div className="modal-image" style={{ backgroundImage: `url(${selectedToken.imageUrl})` }} />
                <div className="modal-body">
                  <h2 className="modal-title">
                    {i18n.language === 'ar' ? selectedToken.nameAr : selectedToken.name}
                  </h2>
                  <div className="modal-meta">
                    <span className="badge-heritage">{selectedToken.type}</span>
                    <span className="badge-heritage">{selectedToken.category}</span>
                    <span className="badge-heritage">📍 {selectedToken.city}</span>
                  </div>
                  <p className="modal-description">{selectedToken.description}</p>
                  {selectedToken.ipfsCid && (
                    <div className="ipfs-info">
                      <h3>IPFS Content</h3>
                      <div className="ipfs-cid monospace">{selectedToken.ipfsCid}</div>
                      <button className="btn-primary">View on IPFS</button>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </Layout>
  );
}
