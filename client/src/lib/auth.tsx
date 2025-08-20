import React, { createContext, useContext, useEffect, useState } from "react";
import { apiRequest } from "./queryClient";

interface User {
  id: string;
  username: string;
  email: string;
  roles: string[];
  firstName?: string;
  lastName?: string;
  profileImageUrl?: string;
}

interface AuthContextType {
  user: User | null;
  isLoading: boolean;
  login: (email: string, password: string) => Promise<{ success: boolean; message?: string }>;
  register: (userData: {
    username: string;
    email: string;
    password: string;
    role?: string;
  }) => Promise<{ success: boolean; message?: string }>;
  logout: () => Promise<void>;
  refreshUser: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const refreshUser = async () => {
    const token = localStorage.getItem("judicia_token");
    if (!token) {
      setIsLoading(false);
      return;
    }

    try {
      const response = await apiRequest("/api/auth/me", {
        headers: { Authorization: `Bearer ${token}` },
      });
      setUser(response.user);
    } catch (error) {
      setUser(null);
      localStorage.removeItem("judicia_token"); // Clean up invalid token
    } finally {
      setIsLoading(false);
    }
  };

  const login = async (email: string, password: string) => {
    try {
      const response = await apiRequest("/api/auth/login", {
        method: "POST",
        body: JSON.stringify({ email, password }),
      });
      if (response.token) {
        localStorage.setItem("judicia_token", response.token);
      }
      setUser(response.user);
      return { success: true };
    } catch (error: any) {
      return { 
        success: false, 
        message: error.message || "Login failed" 
      };
    }
  };

  const register = async (userData: {
    username: string;
    email: string;
    password: string;
    role?: string;
  }) => {
    try {
      const response = await apiRequest("/api/auth/register", {
        method: "POST",
        body: JSON.stringify(userData),
      });
      if (response.token) {
        localStorage.setItem("judicia_token", response.token);
      }
      setUser(response.user);
      return { success: true };
    } catch (error: any) {
      return { 
        success: false, 
        message: error.message || "Registration failed" 
      };
    }
  };

  const logout = async () => {
    try {
      // The backend might not have a logout endpoint for JWT, but we call it just in case.
      // The most important part is clearing client-side state.
      await apiRequest("/api/auth/logout", { method: "POST" });
    } catch (error) {
      // Even if logout fails on server, clear local state
      console.error("Server logout failed, proceeding with client-side logout.", error);
    } finally {
      localStorage.removeItem("judicia_token");
      setUser(null);
    }
  };

  useEffect(() => {
    refreshUser();
  }, []);

  return (
    <AuthContext.Provider value={{
      user,
      isLoading,
      login,
      register,
      logout,
      refreshUser
    }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}

export function RequireAuth({ children, roles }: { 
  children: React.ReactNode; 
  roles?: string[];
}) {
  const { user, isLoading } = useAuth();

  if (isLoading) {
    return <div className="flex items-center justify-center min-h-screen">
      <div className="text-lg">Loading...</div>
    </div>;
  }

  if (!user) {
    return <div className="flex items-center justify-center min-h-screen">
      <div className="text-lg text-red-600">Please log in to access this page</div>
    </div>;
  }

  if (roles && !roles.some(role => user.roles.includes(role))) {
    return <div className="flex items-center justify-center min-h-screen">
      <div className="text-lg text-red-600">Access denied. Insufficient permissions.</div>
    </div>;
  }

  return <>{children}</>;
}

export function RequireGuest({ children }: { children: React.ReactNode }) {
  const { user, isLoading } = useAuth();

  if (isLoading) {
    return <div className="flex items-center justify-center min-h-screen">
      <div className="text-lg">Loading...</div>
    </div>;
  }

  if (user) {
    return <div className="flex items-center justify-center min-h-screen">
      <div className="text-lg">You are already logged in</div>
    </div>;
  }

  return <>{children}</>;
}