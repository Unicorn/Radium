/**
 * Sidebar Navigation Component
 */

'use client';

import { usePathname, useRouter } from 'next/navigation';
import { YStack, XStack, Text, Button, Separator } from 'tamagui';

interface NavItem {
  label: string;
  path: string;
  icon?: string;
}

const navItems: NavItem[] = [
  { label: 'Dashboard', path: '/' },
  { label: 'Projects', path: '/projects' },
  { label: 'Workflows', path: '/workflows' },
  { label: 'Components', path: '/components' },
  { label: 'Agents', path: '/agents' },
];

export function Sidebar() {
  const pathname = usePathname();
  const router = useRouter();

  return (
    <YStack
      width={240}
      backgroundColor="$background"
      borderRightWidth={1}
      borderColor="$borderColor"
      paddingVertical="$4"
    >
      <YStack gap="$2" paddingHorizontal="$2">
        {navItems.map((item) => {
          // Check if current path matches or is a sub-route
          const isActive = 
            pathname === item.path || 
            (item.path !== '/' && pathname.startsWith(item.path));
          
          return (
            <Button
              key={item.path}
              size="$4"
              onPress={() => router.push(item.path)}
              backgroundColor={isActive ? '$blue4' : 'transparent'}
              hoverStyle={{
                backgroundColor: isActive ? '$blue5' : '$gray3',
              }}
              pressStyle={{
                backgroundColor: isActive ? '$blue6' : '$gray4',
              }}
              justifyContent="flex-start"
              paddingHorizontal="$3"
            >
              <Text
                fontSize="$4"
                fontWeight={isActive ? '600' : '400'}
                color={isActive ? '$blue11' : '$color'}
              >
                {item.label}
              </Text>
            </Button>
          );
        })}
      </YStack>
    </YStack>
  );
}

