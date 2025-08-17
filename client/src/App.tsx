import { Switch, Route, Redirect } from "wouter";
import { queryClient } from "./lib/queryClient";
import { QueryClientProvider } from "@tanstack/react-query";
import { AuthProvider, useAuth } from "./lib/auth";
import { Layout } from "@/components/Layout";
import Login from "./pages/Login";
import ContestList from "./pages/ContestList";
import ContestDetail from "./pages/ContestDetail";
import ProblemView from "./pages/ProblemView";
import AdminPanel from "./pages/AdminPanel";
import Profile from "./pages/Profile";
import Settings from "./pages/Settings";
import NotFound from "./pages/not-found";

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { user, isLoading } = useAuth();
  
  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600">Loading...</p>
        </div>
      </div>
    );
  }
  
  if (!user) {
    return <Redirect to="/login" />;
  }
  
  return <>{children}</>;
}

function Router() {
  const { user } = useAuth();
  
  return (
    <Layout>
      <Switch>
        <Route path="/login">
          {user ? <Redirect to="/contests" /> : <Login />}
        </Route>
        
        <Route path="/contests">
          <ProtectedRoute>
            <ContestList />
          </ProtectedRoute>
        </Route>
        
        <Route path="/contests/:id">
          <ProtectedRoute>
            <ContestDetail />
          </ProtectedRoute>
        </Route>
        
        <Route path="/problems/:id">
          <ProtectedRoute>
            <ProblemView />
          </ProtectedRoute>
        </Route>
        
        <Route path="/admin">
          <ProtectedRoute>
            <AdminPanel />
          </ProtectedRoute>
        </Route>
        
        <Route path="/profile">
          <ProtectedRoute>
            <Profile />
          </ProtectedRoute>
        </Route>
        
        <Route path="/settings">
          <ProtectedRoute>
            <Settings />
          </ProtectedRoute>
        </Route>
        
        <Route path="/">
          {user ? <Redirect to="/contests" /> : <Redirect to="/login" />}
        </Route>
        
        <Route component={NotFound} />
      </Switch>
    </Layout>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <Router />
      </AuthProvider>
    </QueryClientProvider>
  );
}

export default App;