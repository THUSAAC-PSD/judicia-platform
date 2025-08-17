import { sql } from "drizzle-orm";
import { pgTable, text, varchar, integer, timestamp, boolean, jsonb, index } from "drizzle-orm/pg-core";
import { createInsertSchema } from "drizzle-zod";
import { z } from "zod";
import { relations } from "drizzle-orm";

// Session storage table for authentication
export const sessions = pgTable(
  "sessions",
  {
    sid: varchar("sid").primaryKey(),
    sess: jsonb("sess").notNull(),
    expire: timestamp("expire").notNull(),
  },
  (table) => [index("IDX_session_expire").on(table.expire)],
);

export const users = pgTable("users", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  username: text("username").notNull().unique(),
  email: text("email").notNull().unique(),
  password: text("password").notNull(),
  role: text("role").notNull().default("contestant"), // contestant, contest_admin, admin, superadmin
  firstName: text("first_name"),
  lastName: text("last_name"),
  profileImageUrl: text("profile_image_url"),
  createdAt: timestamp("created_at").defaultNow(),
  updatedAt: timestamp("updated_at").defaultNow(),
});

export const contests = pgTable("contests", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  title: text("title").notNull(),
  description: text("description"),
  startTime: timestamp("start_time").notNull(),
  endTime: timestamp("end_time").notNull(),
  status: text("status").notNull().default("upcoming"), // upcoming, running, finished
  difficulty: text("difficulty").notNull().default("intermediate"), // beginner, intermediate, advanced, mixed
  maxParticipants: integer("max_participants"),
  createdBy: varchar("created_by").references(() => users.id),
  createdAt: timestamp("created_at").defaultNow(),
});

export const problems = pgTable("problems", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  contestId: varchar("contest_id").references(() => contests.id),
  title: text("title").notNull(),
  statement: text("statement").notNull(),
  difficulty: text("difficulty").notNull().default("easy"), // easy, medium, hard
  timeLimit: integer("time_limit").notNull().default(1000), // milliseconds
  memoryLimit: integer("memory_limit").notNull().default(256), // MB
  points: integer("points").notNull().default(100),
  order: integer("order").notNull().default(0), // A, B, C, etc.
  sampleInput: text("sample_input"),
  sampleOutput: text("sample_output"),
  questionType: text("question_type").notNull().default("standard"), // standard, output-only
  metadata: jsonb("metadata"), // additional question type specific data
  createdAt: timestamp("created_at").defaultNow(),
});

export const submissions = pgTable("submissions", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  problemId: varchar("problem_id").references(() => problems.id),
  userId: varchar("user_id").references(() => users.id),
  contestId: varchar("contest_id").references(() => contests.id),
  code: text("code").notNull(),
  language: text("language").notNull().default("cpp"),
  fileUrl: text("file_url"),
  status: text("status").notNull().default("pending"), // pending, judging, accepted, wrong_answer, time_limit, memory_limit, runtime_error
  score: integer("score").default(0),
  executionTime: integer("execution_time"), // milliseconds
  memoryUsed: integer("memory_used"), // KB
  verdict: text("verdict"),
  submittedAt: timestamp("submitted_at").defaultNow(),
});

export const contestParticipants = pgTable("contest_participants", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  contestId: varchar("contest_id").references(() => contests.id),
  userId: varchar("user_id").references(() => users.id),
  joinedAt: timestamp("joined_at").defaultNow(),
});

export const contestAdmins = pgTable("contest_admins", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  contestId: varchar("contest_id").references(() => contests.id),
  userId: varchar("user_id").references(() => users.id),
  assignedAt: timestamp("assigned_at").defaultNow(),
});

// Insert schemas
export const insertUserSchema = createInsertSchema(users).pick({
  username: true,
  email: true,
  password: true,
  role: true,
  firstName: true,
  lastName: true,
});

export const insertContestSchema = createInsertSchema(contests).pick({
  title: true,
  description: true,
  startTime: true,
  endTime: true,
  difficulty: true,
  maxParticipants: true,
});

export const insertProblemSchema = createInsertSchema(problems).pick({
  contestId: true,
  title: true,
  statement: true,
  difficulty: true,
  timeLimit: true,
  memoryLimit: true,
  points: true,
  order: true,
  sampleInput: true,
  sampleOutput: true,
  questionType: true,
  metadata: true,
});

export const insertSubmissionSchema = createInsertSchema(submissions).pick({
  problemId: true,
  contestId: true,
  code: true,
  language: true,
});

// Login schema
export const loginSchema = z.object({
  email: z.string().email(),
  password: z.string().min(1),
});

// Types
export type InsertUser = z.infer<typeof insertUserSchema>;
export type User = typeof users.$inferSelect;
export type InsertContest = z.infer<typeof insertContestSchema>;
export type Contest = typeof contests.$inferSelect;
export type InsertProblem = z.infer<typeof insertProblemSchema>;
export type Problem = typeof problems.$inferSelect;
export type InsertSubmission = z.infer<typeof insertSubmissionSchema>;
export type Submission = typeof submissions.$inferSelect;
export type Login = z.infer<typeof loginSchema>;
export type ContestParticipant = typeof contestParticipants.$inferSelect;
export type ContestAdmin = typeof contestAdmins.$inferSelect;

// Database relations
export const usersRelations = relations(users, ({ many }) => ({
  contests: many(contests),
  submissions: many(submissions),
  contestParticipants: many(contestParticipants),
  contestAdmins: many(contestAdmins),
}));

export const contestsRelations = relations(contests, ({ one, many }) => ({
  createdBy: one(users, { fields: [contests.createdBy], references: [users.id] }),
  problems: many(problems),
  submissions: many(submissions),
  participants: many(contestParticipants),
  admins: many(contestAdmins),
}));

export const problemsRelations = relations(problems, ({ one, many }) => ({
  contest: one(contests, { fields: [problems.contestId], references: [contests.id] }),
  submissions: many(submissions),
}));

export const submissionsRelations = relations(submissions, ({ one }) => ({
  problem: one(problems, { fields: [submissions.problemId], references: [problems.id] }),
  user: one(users, { fields: [submissions.userId], references: [users.id] }),
  contest: one(contests, { fields: [submissions.contestId], references: [contests.id] }),
}));

export const contestParticipantsRelations = relations(contestParticipants, ({ one }) => ({
  contest: one(contests, { fields: [contestParticipants.contestId], references: [contests.id] }),
  user: one(users, { fields: [contestParticipants.userId], references: [users.id] }),
}));

export const contestAdminsRelations = relations(contestAdmins, ({ one }) => ({
  contest: one(contests, { fields: [contestAdmins.contestId], references: [contests.id] }),
  user: one(users, { fields: [contestAdmins.userId], references: [users.id] }),
}));
