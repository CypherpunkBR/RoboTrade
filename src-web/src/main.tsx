import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { GlobalFilterProvider } from './contexts/GlobalFilterContext'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <GlobalFilterProvider>
      <App />
    </GlobalFilterProvider>
  </StrictMode>,
)
