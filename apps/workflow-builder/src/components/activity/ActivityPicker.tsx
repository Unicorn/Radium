/**
 * Activity Picker Component
 *
 * Modal dialog for selecting activities from the registry.
 * Supports filtering by category, tags, and search.
 */

'use client';

import {
  Dialog,
  YStack,
  XStack,
  Input,
  Button,
  Text,
  Card,
  Spinner,
  Select,
  Adapt,
  Sheet,
  Badge,
} from 'tamagui';
import { useState, useMemo } from 'react';
import { api } from '@/lib/trpc/client';
import { Search, X, Filter, Activity as ActivityIcon } from 'lucide-react';
import type { Database } from '@/types/database';

type Activity = Database['public']['Tables']['activities']['Row'];

interface ActivityPickerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSelect: (activity: Activity) => void;
  selectedActivityId?: string;
}

export function ActivityPicker({
  open,
  onOpenChange,
  onSelect,
  selectedActivityId,
}: ActivityPickerProps) {
  const [search, setSearch] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('');

  // Fetch activities
  const { data: activities, isLoading } = api.activities.list.useQuery(
    {
      search: search || undefined,
      category: selectedCategory || undefined,
    },
    {
      enabled: open, // Only fetch when dialog is open
    }
  );

  // Fetch categories
  const { data: categories } = api.activities.categories.useQuery(undefined, {
    enabled: open,
  });

  // Filter and sort activities
  const filteredActivities = useMemo(() => {
    if (!activities) return [];
    return activities;
  }, [activities]);

  // Group activities by category
  const groupedActivities = useMemo(() => {
    const groups: Record<string, Activity[]> = {};

    filteredActivities.forEach((activity) => {
      const category = activity.category || 'Uncategorized';
      if (!groups[category]) {
        groups[category] = [];
      }
      groups[category].push(activity);
    });

    return groups;
  }, [filteredActivities]);

  const handleClose = () => {
    setSearch('');
    setSelectedCategory('');
    onOpenChange(false);
  };

  const handleSelect = (activity: Activity) => {
    onSelect(activity);
    handleClose();
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange} modal>
      <Dialog.Portal>
        <Dialog.Overlay
          key="overlay"
          animation="quick"
          opacity={0.5}
          enterStyle={{ opacity: 0 }}
          exitStyle={{ opacity: 0 }}
        />
        <Dialog.Content
          key="content"
          bordered
          elevate
          animation={[
            'quick',
            {
              opacity: {
                overshootClamping: true,
              },
            },
          ]}
          enterStyle={{ x: 0, y: -20, opacity: 0, scale: 0.9 }}
          exitStyle={{ x: 0, y: 10, opacity: 0, scale: 0.95 }}
          gap="$4"
          width="90%"
          maxWidth={800}
          maxHeight="80vh"
        >
          <Dialog.Title>Select Activity</Dialog.Title>
          <Dialog.Description>
            Choose an activity to add to your workflow
          </Dialog.Description>

          {/* Filters */}
          <YStack gap="$3">
            {/* Search */}
            <XStack gap="$2" alignItems="center">
              <Search size={20} />
              <Input
                flex={1}
                placeholder="Search activities..."
                value={search}
                onChangeText={setSearch}
              />
              {search && (
                <Button
                  size="$2"
                  circular
                  icon={X}
                  onPress={() => setSearch('')}
                  chromeless
                />
              )}
            </XStack>

            {/* Category Filter */}
            <XStack gap="$2" alignItems="center">
              <Filter size={20} />
              <Select
                value={selectedCategory}
                onValueChange={setSelectedCategory}
                native={false}
              >
                <Select.Trigger flex={1}>
                  <Select.Value placeholder="All Categories" />
                </Select.Trigger>

                <Adapt when="sm" platform="touch">
                  <Sheet
                    modal
                    dismissOnSnapToBottom
                    animationConfig={{
                      type: 'spring',
                      damping: 20,
                      mass: 1.2,
                      stiffness: 250,
                    }}
                  >
                    <Sheet.Frame>
                      <Sheet.ScrollView>
                        <Adapt.Contents />
                      </Sheet.ScrollView>
                    </Sheet.Frame>
                    <Sheet.Overlay
                      animation="lazy"
                      enterStyle={{ opacity: 0 }}
                      exitStyle={{ opacity: 0 }}
                    />
                  </Sheet>
                </Adapt>

                <Select.Content zIndex={200000}>
                  <Select.Viewport minWidth={200}>
                    <Select.Group>
                      <Select.Label>Category</Select.Label>
                      <Select.Item index={0} value="">
                        <Select.ItemText>All Categories</Select.ItemText>
                      </Select.Item>
                      {categories?.map((category, i) => (
                        <Select.Item
                          key={category.id}
                          index={i + 1}
                          value={category.name}
                        >
                          <Select.ItemText>{category.name}</Select.ItemText>
                        </Select.Item>
                      ))}
                    </Select.Group>
                  </Select.Viewport>
                </Select.Content>
              </Select>
            </XStack>
          </YStack>

          {/* Activity List */}
          <YStack
            gap="$2"
            maxHeight={400}
            overflow="scroll"
            paddingHorizontal="$2"
          >
            {isLoading ? (
              <XStack justifyContent="center" padding="$4">
                <Spinner size="large" />
              </XStack>
            ) : filteredActivities.length === 0 ? (
              <Card padding="$4">
                <Text textAlign="center" color="$gray11">
                  No activities found
                </Text>
              </Card>
            ) : (
              Object.entries(groupedActivities).map(([category, items]) => (
                <YStack key={category} gap="$2">
                  <Text
                    fontSize="$3"
                    fontWeight="600"
                    color="$gray11"
                    paddingTop="$2"
                  >
                    {category}
                  </Text>
                  {items.map((activity) => (
                    <Card
                      key={activity.id}
                      padding="$3"
                      pressStyle={{
                        backgroundColor: '$blue2',
                        borderColor: '$blue6',
                      }}
                      hoverStyle={{
                        backgroundColor: '$gray3',
                        borderColor: '$gray6',
                      }}
                      onPress={() => handleSelect(activity)}
                      cursor="pointer"
                      backgroundColor={
                        selectedActivityId === activity.id ? '$blue2' : undefined
                      }
                      borderColor={
                        selectedActivityId === activity.id ? '$blue6' : undefined
                      }
                    >
                      <XStack gap="$3" alignItems="flex-start">
                        <YStack
                          backgroundColor="$blue4"
                          padding="$2"
                          borderRadius="$2"
                          alignSelf="flex-start"
                        >
                          <ActivityIcon size={20} />
                        </YStack>

                        <YStack flex={1} gap="$1">
                          <XStack justifyContent="space-between" alignItems="center">
                            <Text fontWeight="600" fontSize="$4">
                              {activity.name}
                            </Text>
                            {activity.usage_count > 0 && (
                              <Badge size="$1">
                                {activity.usage_count} uses
                              </Badge>
                            )}
                          </XStack>

                          {activity.description && (
                            <Text color="$gray11" fontSize="$2">
                              {activity.description}
                            </Text>
                          )}

                          {/* Tags */}
                          {activity.tags && activity.tags.length > 0 && (
                            <XStack gap="$1" flexWrap="wrap" marginTop="$1">
                              {activity.tags.slice(0, 3).map((tag) => (
                                <Badge key={tag} size="$1" backgroundColor="$gray4">
                                  {tag}
                                </Badge>
                              ))}
                              {activity.tags.length > 3 && (
                                <Badge size="$1" backgroundColor="$gray4">
                                  +{activity.tags.length - 3}
                                </Badge>
                              )}
                            </XStack>
                          )}

                          {/* Package info */}
                          <Text color="$gray10" fontSize="$1" marginTop="$1">
                            {activity.package_name}
                          </Text>
                        </YStack>
                      </XStack>
                    </Card>
                  ))}
                </YStack>
              ))
            )}
          </YStack>

          {/* Actions */}
          <XStack gap="$3" justifyContent="flex-end">
            <Dialog.Close asChild>
              <Button chromeless onPress={handleClose}>
                Cancel
              </Button>
            </Dialog.Close>
          </XStack>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog>
  );
}
