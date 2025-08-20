import session from "express-session";
import connectPg from "connect-pg-simple";
import { Request, Response, NextFunction } from "express";
import { storage } from "./storage";
import { loginSchema } from "@shared/schema";

// Session configuration
export function getSession() {
  const sessionTtl = 7 * 24 * 60 * 60 * 1000; // 1 week
  const pgStore = connectPg(session);
  const sessionStore = new pgStore({
    conString: process.env.DATABASE_URL,
    createTableIfMissing: false,
    ttl: sessionTtl,
    tableName: "sessions",
  });
  
  return session({
    secret: process.env.SESSION_SECRET!,
    store: sessionStore,
    resave: false,
    saveUninitialized: false,
    cookie: {
      httpOnly: true,
  secure: process.env.NODE_ENV === "production",
  sameSite: process.env.NODE_ENV === "production" ? "lax" : "lax",
      maxAge: sessionTtl,
    },
  });
}

// Authentication middleware
export const requireAuth = async (req: Request, res: Response, next: NextFunction) => {
  if (!req.session?.userId) {
    return res.status(401).json({ message: "Authentication required" });
  }
  
  try {
    const user = await storage.getUser(req.session.userId);
    if (!user) {
      req.session.destroy(() => {});
      return res.status(401).json({ message: "User not found" });
    }
    
    req.user = user;
    next();
  } catch (error) {
    return res.status(500).json({ message: "Authentication error" });
  }
};

// Admin middleware
export const requireAdmin = async (req: Request, res: Response, next: NextFunction) => {
  await requireAuth(req, res, () => {
    if (!req.user || (req.user.role !== "admin" && req.user.role !== "superadmin")) {
      return res.status(403).json({ message: "Admin access required" });
    }
    next();
  });
};

// Contest Admin middleware - can manage their assigned contests
export const requireContestAdmin = async (req: Request, res: Response, next: NextFunction) => {
  await requireAuth(req, res, () => {
    if (!req.user || (req.user.role !== "contest_admin" && req.user.role !== "admin" && req.user.role !== "superadmin")) {
      return res.status(403).json({ message: "Contest admin access required" });
    }
    next();
  });
};

// Contest Admin or higher middleware
export const requireContestAdminOrHigher = async (req: Request, res: Response, next: NextFunction) => {
  await requireAuth(req, res, () => {
    if (!req.user || !["contest_admin", "admin", "superadmin"].includes(req.user.role)) {
      return res.status(403).json({ message: "Contest admin access or higher required" });
    }
    next();
  });
};

// Middleware to check if user can manage a specific contest
export const requireContestManagePermission = async (req: Request, res: Response, next: NextFunction) => {
  await requireAuth(req, res, async () => {
    const user = req.user!;
    const contestId = req.params.id || req.params.contestId;
    
    if (!contestId) {
      return res.status(400).json({ message: "Contest ID is required" });
    }

    // Superadmin and admin can manage any contest
    if (user.role === "superadmin" || user.role === "admin") {
      return next();
    }

    // Contest admin can only manage their assigned contests
    if (user.role === "contest_admin") {
      try {
        const { storage } = await import("./storage");
        const canManage = await storage.isUserContestAdmin(contestId, user.id);
        if (canManage) {
          return next();
        }
      } catch (error) {
        return res.status(500).json({ message: "Error checking permissions" });
      }
    }

    return res.status(403).json({ message: "You don't have permission to manage this contest" });
  });
};

// SuperAdmin middleware
export const requireSuperAdmin = async (req: Request, res: Response, next: NextFunction) => {
  await requireAuth(req, res, () => {
    if (!req.user || req.user.role !== "superadmin") {
      return res.status(403).json({ message: "Superadmin access required" });
    }
    next();
  });
};

// Extend Express session and request types
declare module "express-session" {
  interface SessionData {
    userId?: string;
  }
}

declare global {
  namespace Express {
    interface Request {
      user?: {
        id: string;
        username: string;
        email: string;
        role: string;
        firstName?: string | null;
        lastName?: string | null;
        profileImageUrl?: string | null;
        createdAt?: Date | null;
        updatedAt?: Date | null;
      };
    }
  }
}