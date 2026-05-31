import { createRoot } from 'react-dom/client'
import { HashRouter } from 'react-router'
import './index.css'
import './components/docx-editor/docx-editor-theme-overrides.css'
import App from './App.tsx'
import { initThemeBeforeRender } from './context/ThemeProvider'

initThemeBeforeRender()

createRoot(document.getElementById('root')!).render(
  <HashRouter>
    <App />
  </HashRouter>,
)
