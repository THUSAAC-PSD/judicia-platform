import {
  users,
  contests,
  problems,
  submissions,
  contestParticipants,
  contestAdmins,
  type User,
  type Contest,
  type Problem,
  type Submission,
  type ContestParticipant,
  type ContestAdmin,
  type InsertUser,
  type InsertContest,
  type InsertProblem,
  type InsertSubmission,
} from "@shared/schema";
import { db } from "./db";
import { eq, desc, and } from "drizzle-orm";
import bcrypt from "bcrypt";

export interface IStorage {
  // Authentication
  authenticateUser(email: string, password: string): Promise<User | null>;
  
  // Users
  getUser(id: string): Promise<User | undefined>;
  getUserByEmail(email: string): Promise<User | undefined>;
  getUserByUsername(username: string): Promise<User | undefined>;
  createUser(user: InsertUser): Promise<User>;
  updateUser(id: string, updates: Partial<User>): Promise<User | undefined>;
  deleteUser(id: string): Promise<boolean>;
  getAllUsers(): Promise<User[]>;
  
  // Contests
  getContests(): Promise<Contest[]>;
  getContest(id: string): Promise<Contest | undefined>;
  createContest(contest: InsertContest & { createdBy: string }): Promise<Contest>;
  updateContest(id: string, updates: Partial<Contest>): Promise<Contest | undefined>;
  deleteContest(id: string): Promise<boolean>;
  getUserContests(userId: string): Promise<Contest[]>;
  
  // Problems
  getContestProblems(contestId: string): Promise<Problem[]>;
  getProblem(id: string): Promise<Problem | undefined>;
  createProblem(problem: InsertProblem): Promise<Problem>;
  updateProblem(id: string, updates: Partial<Problem>): Promise<Problem | undefined>;
  deleteProblem(id: string): Promise<boolean>;
  getAllProblems(): Promise<Problem[]>;
  
  // Submissions
  getSubmissions(userId: string, contestId?: string): Promise<Submission[]>;
  getAllSubmissions(): Promise<Submission[]>;
  getProblemSubmissions(problemId: string, userId: string): Promise<Submission[]>;
  createSubmission(submission: InsertSubmission & { userId: string }): Promise<Submission>;
  updateSubmission(id: string, updates: Partial<Submission>): Promise<Submission | undefined>;
  deleteSubmission(id: string): Promise<boolean>;
  
  // Contest Participants
  joinContest(contestId: string, userId: string): Promise<ContestParticipant>;
  getContestParticipants(contestId: string): Promise<ContestParticipant[]>;
  isUserInContest(contestId: string, userId: string): Promise<boolean>;
  leaveContest(contestId: string, userId: string): Promise<boolean>;
  
  // Contest Admins
  assignContestAdmin(contestId: string, userId: string): Promise<ContestAdmin>;
  removeContestAdmin(contestId: string, userId: string): Promise<boolean>;
  getContestAdmins(contestId: string): Promise<User[]>;
  getUserAdminContests(userId: string): Promise<Contest[]>;
  isUserContestAdmin(contestId: string, userId: string): Promise<boolean>;
}

export class DatabaseStorage implements IStorage {
  constructor() {
    this.initializeSuperAdmin();
  }

  private async initializeSuperAdmin() {
    try {
      const superAdminEmail = process.env.SUPERADMIN_EMAIL;
      const superAdminPassword = process.env.SUPERADMIN_PASSWORD;
      
      if (!superAdminEmail || !superAdminPassword) {
        console.warn("Superadmin credentials not found in environment variables");
        return;
      }

      // Check if superadmin already exists
      const existingAdmin = await this.getUserByEmail(superAdminEmail);
      if (existingAdmin) {
        return;
      }

      // Create superadmin user
      const hashedPassword = await bcrypt.hash(superAdminPassword, 10);
      await db.insert(users).values({
        email: superAdminEmail,
        username: "superadmin",
        password: hashedPassword,
        role: "superadmin",
        firstName: "Super",
        lastName: "Admin",
      });

      console.log("Superadmin user created successfully");
    } catch (error) {
      console.error("Failed to initialize superadmin:", error);
    }
  }

  // Authentication
  async authenticateUser(email: string, password: string): Promise<User | null> {
    const user = await this.getUserByEmail(email);
    if (!user) return null;

    const isValid = await bcrypt.compare(password, user.password);
    return isValid ? user : null;
  }

  // Users
  async getUser(id: string): Promise<User | undefined> {
    const [user] = await db.select().from(users).where(eq(users.id, id));
    return user;
  }

  async getUserByEmail(email: string): Promise<User | undefined> {
    const [user] = await db.select().from(users).where(eq(users.email, email));
    return user;
  }

  async getUserByUsername(username: string): Promise<User | undefined> {
    const [user] = await db.select().from(users).where(eq(users.username, username));
    return user;
  }

  async createUser(userData: InsertUser): Promise<User> {
    const hashedPassword = await bcrypt.hash(userData.password, 10);
    const [user] = await db
      .insert(users)
      .values({
        ...userData,
        password: hashedPassword,
      })
      .returning();
    return user;
  }

  async updateUser(id: string, updates: Partial<User>): Promise<User | undefined> {
    if (updates.password) {
      updates.password = await bcrypt.hash(updates.password, 10);
    }
    const [user] = await db
      .update(users)
      .set({ ...updates, updatedAt: new Date() })
      .where(eq(users.id, id))
      .returning();
    return user;
  }

  async deleteUser(id: string): Promise<boolean> {
    const result = await db.delete(users).where(eq(users.id, id));
    return (result.rowCount ?? 0) > 0;
  }

  async getAllUsers(): Promise<User[]> {
    return await db.select().from(users).orderBy(desc(users.createdAt));
  }

  // Contests
  async getContests(): Promise<Contest[]> {
    return await db.select().from(contests).orderBy(desc(contests.createdAt));
  }

  async getContest(id: string): Promise<Contest | undefined> {
    const [contest] = await db.select().from(contests).where(eq(contests.id, id));
    return contest;
  }

  async createContest(contestData: InsertContest & { createdBy: string }): Promise<Contest> {
    const [contest] = await db
      .insert(contests)
      .values(contestData)
      .returning();
    return contest;
  }

  async updateContest(id: string, updates: Partial<Contest>): Promise<Contest | undefined> {
    const [contest] = await db
      .update(contests)
      .set(updates)
      .where(eq(contests.id, id))
      .returning();
    return contest;
  }

  async deleteContest(id: string): Promise<boolean> {
    const result = await db.delete(contests).where(eq(contests.id, id));
    return (result.rowCount ?? 0) > 0;
  }

  async getUserContests(userId: string): Promise<Contest[]> {
    const userParticipations = await db
      .select({ contestId: contestParticipants.contestId })
      .from(contestParticipants)
      .where(eq(contestParticipants.userId, userId));
    
    if (userParticipations.length === 0) return [];
    
    const contestIds = userParticipations.map(p => p.contestId);
    // For now, just return all contests since the user isn't in any specific contests yet
    return await db.select().from(contests).orderBy(desc(contests.createdAt));
  }

  // Problems
  async getContestProblems(contestId: string): Promise<Problem[]> {
    return await db
      .select()
      .from(problems)
      .where(eq(problems.contestId, contestId))
      .orderBy(problems.order);
  }

  async getProblem(id: string): Promise<Problem | undefined> {
    const [problem] = await db.select().from(problems).where(eq(problems.id, id));
    return problem;
  }

  async createProblem(problemData: InsertProblem): Promise<Problem> {
    const [problem] = await db
      .insert(problems)
      .values(problemData)
      .returning();
    return problem;
  }

  async updateProblem(id: string, updates: Partial<Problem>): Promise<Problem | undefined> {
    const [problem] = await db
      .update(problems)
      .set(updates)
      .where(eq(problems.id, id))
      .returning();
    return problem;
  }

  async deleteProblem(id: string): Promise<boolean> {
    const result = await db.delete(problems).where(eq(problems.id, id));
    return (result.rowCount ?? 0) > 0;
  }

  async getAllProblems(): Promise<Problem[]> {
    return await db.select().from(problems).orderBy(desc(problems.createdAt));
  }

  // Submissions
  async getSubmissions(userId: string, contestId?: string): Promise<Submission[]> {
    if (contestId) {
      return await db
        .select()
        .from(submissions)
        .where(and(eq(submissions.userId, userId), eq(submissions.contestId, contestId)))
        .orderBy(desc(submissions.submittedAt));
    }
    
    return await db
      .select()
      .from(submissions)
      .where(eq(submissions.userId, userId))
      .orderBy(desc(submissions.submittedAt));
  }

  async getAllSubmissions(): Promise<Submission[]> {
    return await db.select().from(submissions).orderBy(desc(submissions.submittedAt));
  }

  async getProblemSubmissions(problemId: string, userId: string): Promise<Submission[]> {
    return await db
      .select()
      .from(submissions)
      .where(and(eq(submissions.problemId, problemId), eq(submissions.userId, userId)))
      .orderBy(desc(submissions.submittedAt));
  }

  async createSubmission(submissionData: InsertSubmission & { userId: string }): Promise<Submission> {
    const [submission] = await db
      .insert(submissions)
      .values(submissionData)
      .returning();
    return submission;
  }

  async updateSubmission(id: string, updates: Partial<Submission>): Promise<Submission | undefined> {
    const [submission] = await db
      .update(submissions)
      .set(updates)
      .where(eq(submissions.id, id))
      .returning();
    return submission;
  }

  async deleteSubmission(id: string): Promise<boolean> {
    const result = await db.delete(submissions).where(eq(submissions.id, id));
    return (result.rowCount ?? 0) > 0;
  }

  // Contest Participants
  async joinContest(contestId: string, userId: string): Promise<ContestParticipant> {
    const [participation] = await db
      .insert(contestParticipants)
      .values({ contestId, userId })
      .returning();
    return participation;
  }

  async getContestParticipants(contestId: string): Promise<ContestParticipant[]> {
    return await db
      .select()
      .from(contestParticipants)
      .where(eq(contestParticipants.contestId, contestId));
  }

  async isUserInContest(contestId: string, userId: string): Promise<boolean> {
    const [participant] = await db
      .select()
      .from(contestParticipants)
      .where(and(eq(contestParticipants.contestId, contestId), eq(contestParticipants.userId, userId)));
    return !!participant;
  }

  async leaveContest(contestId: string, userId: string): Promise<boolean> {
    const result = await db
      .delete(contestParticipants)
      .where(and(eq(contestParticipants.contestId, contestId), eq(contestParticipants.userId, userId)));
    return (result.rowCount ?? 0) > 0;
  }

  // Contest Admins
  async assignContestAdmin(contestId: string, userId: string): Promise<ContestAdmin> {
    const [assignment] = await db
      .insert(contestAdmins)
      .values({ contestId, userId })
      .returning();
    return assignment;
  }

  async removeContestAdmin(contestId: string, userId: string): Promise<boolean> {
    const result = await db
      .delete(contestAdmins)
      .where(and(eq(contestAdmins.contestId, contestId), eq(contestAdmins.userId, userId)));
    return (result.rowCount ?? 0) > 0;
  }

  async getContestAdmins(contestId: string): Promise<User[]> {
    const result = await db
      .select({
        id: users.id,
        username: users.username,
        email: users.email,
        password: users.password,
        role: users.role,
        firstName: users.firstName,
        lastName: users.lastName,
        profileImageUrl: users.profileImageUrl,
        createdAt: users.createdAt,
        updatedAt: users.updatedAt,
      })
      .from(contestAdmins)
      .innerJoin(users, eq(contestAdmins.userId, users.id))
      .where(eq(contestAdmins.contestId, contestId));
    return result;
  }

  async getUserAdminContests(userId: string): Promise<Contest[]> {
    const result = await db
      .select({
        id: contests.id,
        title: contests.title,
        description: contests.description,
        startTime: contests.startTime,
        endTime: contests.endTime,
        status: contests.status,
        difficulty: contests.difficulty,
        maxParticipants: contests.maxParticipants,
        createdBy: contests.createdBy,
        createdAt: contests.createdAt,
      })
      .from(contestAdmins)
      .innerJoin(contests, eq(contestAdmins.contestId, contests.id))
      .where(eq(contestAdmins.userId, userId));
    return result;
  }

  async isUserContestAdmin(contestId: string, userId: string): Promise<boolean> {
    const [admin] = await db
      .select()
      .from(contestAdmins)
      .where(and(eq(contestAdmins.contestId, contestId), eq(contestAdmins.userId, userId)));
    return !!admin;
  }
}

export const storage = new DatabaseStorage();