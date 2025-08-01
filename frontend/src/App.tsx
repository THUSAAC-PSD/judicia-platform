import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { createBrowserRouter, RouterProvider } from 'react-router-dom'
import { Dashboard } from './pages/Dashboard'
import { Problems } from './pages/Problems'
import { Contests } from './pages/Contests'

// Create a new query client
const queryClient = new QueryClient()

// Create router
const router = createBrowserRouter([
  {
    path: '/',
    element: <Dashboard />,
  },
  {
    path: '/problems',
    element: <Problems />,
  },
  {
    path: '/contests',
    element: <Contests />,
  },
])

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  )
}

export default App
