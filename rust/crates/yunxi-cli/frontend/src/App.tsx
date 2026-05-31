import { Routes, Route } from 'react-router'
import Login from './pages/Login'
import Onboarding from './pages/Onboarding'
import MainApp from './pages/MainApp'
import Settings from './pages/Settings'
import Layout from './components/Layout'
import { AppProvider } from './context/AppProvider'
import { ErrorBoundary } from './components/ErrorBoundary'
import { ThemeProvider } from './context/ThemeProvider'
import AppearanceBridge from './components/AppearanceBridge'
import OnboardingGuard from './components/OnboardingGuard'

export default function App() {
  return (
    <ErrorBoundary>
      <ThemeProvider>
        <AppProvider>
          <AppearanceBridge />
          <OnboardingGuard />
          <Routes>
            <Route path="/" element={<MainApp />} />
            <Route path="/onboarding" element={<Onboarding />} />
            <Route path="/login" element={<Login />} />
            <Route path="/settings" element={<Layout contentMode="full"><Settings /></Layout>} />
          </Routes>
        </AppProvider>
      </ThemeProvider>
    </ErrorBoundary>
  )
}
