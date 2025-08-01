// Core types for the online judge system

export interface Problem {
  id: string;
  title: string;
  description: string;
  difficulty: 'Easy' | 'Medium' | 'Hard';
  timeLimit: number; // in seconds
  memoryLimit: number; // in MB
  tags: string[];
  sampleInput: string;
  sampleOutput: string;
  createdAt: string;
  updatedAt: string;
}

export interface Contest {
  id: string;
  name: string;
  description: string;
  contestType: 'IOI' | 'ICPC' | 'AtCoder' | 'Custom';
  startTime: string;
  endTime: string;
  problems: Problem[];
  participants: User[];
  status: 'Draft' | 'Scheduled' | 'Running' | 'Finished';
  isPublic: boolean;
}

export interface User {
  id: string;
  username: string;
  email: string;
  name: string;
  role: 'Admin' | 'Judge' | 'Contestant';
  avatar?: string;
  rating?: number;
  createdAt: string;
}

export interface Submission {
  id: string;
  problemId: string;
  contestId?: string;
  userId: string;
  language: string;
  sourceCode: string;
  status: 'Pending' | 'Judging' | 'Accepted' | 'Wrong Answer' | 'Time Limit Exceeded' | 'Memory Limit Exceeded' | 'Runtime Error' | 'Compilation Error';
  score?: number;
  executionTime?: number; // in ms
  memoryUsed?: number; // in KB
  submittedAt: string;
  judgedAt?: string;
}

export interface TestCase {
  id: string;
  problemId: string;
  input: string;
  expectedOutput: string;
  isPublic: boolean;
  points: number;
}

export interface Leaderboard {
  contestId: string;
  standings: Standing[];
  lastUpdated: string;
}

export interface Standing {
  rank: number;
  user: User;
  totalScore: number;
  penalty: number; // in minutes for ICPC style
  problemResults: ProblemResult[];
}

export interface ProblemResult {
  problemId: string;
  attempts: number;
  score: number;
  solved: boolean;
  firstSolveTime?: number; // minutes from contest start
}
