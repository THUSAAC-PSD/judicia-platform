# Supabase Configuration Guide

This guide will help you configure the Judicia Platform to use Supabase as your PostgreSQL database provider.

## Prerequisites

- A Supabase account (free tier available)
- Node.js and pnpm installed
- Rust toolchain installed

## Step 1: Create a Supabase Project

1. Go to [supabase.com](https://supabase.com) and sign up/sign in
2. Click "New Project" 
3. Choose your organization
4. Set project name: `judicia-platform`
5. Create a secure database password (save this!)
6. Select a region close to your users
7. Click "Create new project"

Wait for the project to be provisioned (usually 1-2 minutes).

## Step 2: Get Your Supabase Credentials

Once your project is ready:

1. Go to **Settings** → **Database**
2. Find the **Connection string** section
3. Copy the **URI** connection string (it looks like):
   ```
   postgresql://postgres.abcdefghijklmnop:[YOUR-PASSWORD]@aws-0-us-west-1.pooler.supabase.com:5432/postgres
   ```

4. Go to **Settings** → **API**
5. Copy these values:
   - **Project URL**: `https://abcdefghijklmnop.supabase.co`
   - **anon public key**: `eyJ...` (long token)
   - **service_role secret key**: `eyJ...` (long token, keep this secure!)

## Step 3: Configure Environment Variables

1. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` and update these values:

   ```env
   # Database Configuration (Option 1 - Recommended)
   DATABASE_URL=postgresql://postgres.your-project-id:[YOUR-PASSWORD]@aws-0-[region].pooler.supabase.com:5432/postgres
   
   # Supabase Configuration (Option 2 - Alternative)
   SUPABASE_URL=https://your-project-id.supabase.co
   SUPABASE_ANON_KEY=your-anon-key
   SUPABASE_SERVICE_ROLE_KEY=your-service-role-key
   
   # Other required settings
   JWT_SECRET=your-secure-jwt-secret-here
   SERVER_ADDRESS=127.0.0.1:5000
   ```

3. Replace placeholders with your actual values:
   - `your-project-id`: Your Supabase project ID
   - `[YOUR-PASSWORD]`: Your database password
   - `[region]`: Your chosen region (e.g., `us-west-1`)
   - `your-anon-key`: Your anon public key
   - `your-service-role-key`: Your service role secret key

## Step 4: Run Database Migrations

The Judicia Platform will automatically run migrations when it starts. To run them manually:

```bash
# Install dependencies
pnpm install

# Run migrations
pnpm run db:migrate
```

Or run migrations directly:
```bash
cargo run -p core-kernel -- migrate
```

## Step 5: Start the Platform

Start the development server:

```bash
# Start both frontend and backend
pnpm run dev

# Or start them separately
pnpm run dev:frontend  # Vite dev server
pnpm run dev:backend   # Rust core-kernel
```

## Verification

1. Check that the application starts without database errors
2. Try creating a user account through the UI
3. Verify data appears in your Supabase dashboard under **Table Editor**

## Supabase Dashboard Features

Your Supabase project provides:

### Database Management
- **Table Editor**: View and edit your data
- **SQL Editor**: Run custom queries
- **Database**: Manage schemas, functions, triggers

### Authentication (Optional)
- **Authentication**: User management (can integrate with Judicia's auth)
- **Policies**: Row Level Security (RLS) policies

### API & Monitoring
- **API Docs**: Auto-generated API documentation
- **Logs**: View database and API logs
- **Monitoring**: Performance metrics

## Production Considerations

### Security
1. **Enable Row Level Security (RLS)**:
   ```sql
   ALTER TABLE users ENABLE ROW LEVEL SECURITY;
   ALTER TABLE contests ENABLE ROW LEVEL SECURITY;
   -- Add policies as needed
   ```

2. **Restrict API access**:
   - Go to **Settings** → **API**
   - Configure **CORS settings** for your domain
   - Set up **JWT custom claims** if needed

### Performance
1. **Connection Pooling**: Supabase automatically handles this
2. **Database Optimization**:
   - Monitor slow queries in the dashboard
   - Add indexes for frequently queried columns
   - Use prepared statements (SQLx does this automatically)

### Scaling
1. **Plan Limits**: Monitor your plan usage in the dashboard
2. **Read Replicas**: Available on Pro plan and above
3. **Point-in-time Recovery**: Configure backup retention

## Troubleshooting

### Common Issues

**Connection refused**:
- Check your `DATABASE_URL` format
- Verify your password doesn't contain special characters that need URL encoding
- Ensure your Supabase project is not paused

**Migration errors**:
- Check if tables already exist in your Supabase database
- Review migration files in `core-kernel/migrations/`
- Run `cargo run -p core-kernel -- migrate` to see detailed errors

**Authentication issues**:
- Verify your `JWT_SECRET` is set and consistent
- Check that your Supabase service role key is correct
- Ensure your project is using the correct region

### Getting Help

1. **Supabase Status**: Check [status.supabase.com](https://status.supabase.com)
2. **Supabase Docs**: [supabase.com/docs](https://supabase.com/docs)
3. **Community**: [discord.gg/supabase](https://discord.gg/supabase)

## Alternative Services

If you prefer other hosted PostgreSQL services:

- **Neon**: Similar setup, just replace the `DATABASE_URL`
- **Railway**: PostgreSQL addon with simple setup
- **Heroku Postgres**: Reliable but more expensive
- **AWS RDS**: Full control but more complex setup

The Judicia Platform works with any PostgreSQL 13+ database that supports the required extensions.

## Local Development Fallback

To use local PostgreSQL instead of Supabase:

```bash
# Use the local development script
pnpm run dev:local

# Or set environment manually
DATABASE_URL="postgresql://localhost/judicia" pnpm run dev:backend
```