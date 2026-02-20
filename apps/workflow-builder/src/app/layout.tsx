import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import { TRPCProvider } from '@/lib/trpc/Provider';
import { TamaguiProvider } from '@/lib/tamagui/Provider';
import '@tamagui/core/reset.css';
import '@/styles/accessibility.css';

const inter = Inter({ subsets: ['latin'] });

export const metadata: Metadata = {
  title: 'Workflow Builder',
  description: 'Build and deploy Temporal workflows visually',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={inter.className} suppressHydrationWarning>
        {/* Skip to main content for keyboard navigation */}
        <a href="#main-content" className="skip-to-main">
          Skip to main content
        </a>
        <TRPCProvider>
          <TamaguiProvider>
            {children}
          </TamaguiProvider>
        </TRPCProvider>
      </body>
    </html>
  );
}

