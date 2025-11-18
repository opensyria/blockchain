import { useEffect } from 'react';
import { useRegisterSW } from 'virtual:pwa-register/react';

export function PWABadge() {
  const {
    offlineReady: [offlineReady, setOfflineReady],
    needRefresh: [needRefresh, setNeedRefresh],
    updateServiceWorker,
  } = useRegisterSW({
    onRegistered(r: any) {
      console.log('SW Registered: ' + r);
    },
    onRegisterError(error: any) {
      console.log('SW registration error', error);
    },
  });

  const close = () => {
    setOfflineReady(false);
    setNeedRefresh(false);
  };

  useEffect(() => {
    if (offlineReady) {
      console.log('App ready to work offline');
    }
  }, [offlineReady]);

  return (
    <>
      {(offlineReady || needRefresh) && (
        <div className="pwa-toast">
          <div className="pwa-message">
            {offlineReady ? (
              <span>App ready to work offline</span>
            ) : (
              <span>New content available, click reload to update.</span>
            )}
          </div>
          <div className="pwa-buttons">
            {needRefresh && (
              <button className="pwa-reload-btn" onClick={() => updateServiceWorker(true)}>
                Reload
              </button>
            )}
            <button className="pwa-close-btn" onClick={() => close()}>
              Close
            </button>
          </div>
        </div>
      )}
    </>
  );
}
