import { useState, useEffect } from 'react'
import { fetchActivities } from './api/activities'
import { BullSharkActivity } from './types/activity'
import { Leaderboard } from './components/Leaderboard'
import bullsharksLogo from '../assets/bullsharks_logo_t.png'
import './App.css'

function App() {
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [activities, setActivities] = useState<BullSharkActivity[]>([])

  useEffect(() => {
    const loadActivities = async () => {
      setIsLoading(true)
      setError(null)

      try {
        const data = await fetchActivities()
        setActivities(data)
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load activities')
        console.error('Error loading activities:', err)
      } finally {
        setIsLoading(false)
      }
    }

    loadActivities()
  }, [])

  return (
    <div className="app">
      <div className="container">
        <img src={bullsharksLogo} alt="Bullsharks Logo" className="logo" />
        
        {isLoading ? (
          <div className="loading-message">
            <p>Reeling in your data...</p>
          </div>
        ) : error ? (
          <div className="error-message">
            <p>Oops! Something went wrong: {error}</p>
            <button onClick={() => window.location.reload()} className="cta-button">
              Try Again
            </button>
          </div>
        ) : (
          <>
            <Leaderboard activities={activities} />
          </>
        )}
      </div>
    </div>
  )
}

export default App
