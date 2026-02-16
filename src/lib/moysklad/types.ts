// Типы для API МойСклад

export interface Meta {
  href: string;
  metadataHref?: string;
  type: string;
  mediaType: string;
}

export interface EntityRef {
  meta: Meta;
  id: string;
  name: string;
}

export interface StockStore {
  meta: Meta;
  id: string;
  name: string;
}

export interface Product {
  meta: Meta;
  id: string;
  name: string;
  code?: string;
  externalCode?: string;
  archiving?: string;
  pathName?: string;
  uom?: EntityRef;
  salePrices?: Array<{
    value: number;
    currency: EntityRef;
    priceType: EntityRef;
  }>;
  buyPrice?: {
    value: number;
    currency: EntityRef;
  };
  attributes?: Array<Attribute>;
}

export interface Attribute {
  id: string;
  name: string;
  type: string;
  value?: string | number | boolean | EntityRef;
  meta?: Meta;
}

export interface StockRow {
  meta?: Meta;
  stockByStore?: Array<{
    meta: Meta;
    stock: number;
    reserve: number;
    inTransit: number;
  }>;
  stock?: number;
  reserve?: number;
  inTransit?: number;
  name?: string;
  code?: string;
  article?: string;
  assortmentId: string;
  variantId?: string;
}

export interface ProcessingPlan {
  meta: Meta;
  id: string;
  name: string;
  externalCode?: string;
  archiving?: string;
  products: Array<{
    meta: Meta;
    id: string;
    name: string;
    quantity: number;
  }>;
  materials: Array<{
    meta: Meta;
    id: string;
    name: string;
    quantity: number;
  }>;
  stages?: Array<{
    id: string;
    name: string;
    status: string;
    processingPlanPositions: Array<{
      id: string;
      quantity: number;
      productId: string;
    }>;
  }>;
}

export interface Processing {
  meta: Meta;
  id: string;
  name: string;
  description?: string;
  externalCode?: string;
  moment: string;
  applicable: boolean;
  status: string;
  processingPlan: EntityRef;
  products: ProcessingProduct[];
  materials: ProcessingMaterial[];
  store: EntityRef;
  organization: EntityRef;
  created: string;
  updated: string;
  printed?: boolean;
  published?: boolean;
}

export interface ProcessingProduct {
  id?: string;
  meta?: Meta;
  processingPlanPosition?: {
    meta: Meta;
    id: string;
    quantity: number;
  };
  processingPlanProduct?: {
    meta: Meta;
    id: string;
    name: string;
  };
  assortment: EntityRef;
  product: EntityRef;
  quantity: number;
  quantityPerProduct?: number;
}

export interface ProcessingMaterial {
  id?: string;
  meta?: Meta;
  processingPlanPosition?: {
    meta: Meta;
    id: string;
    quantity: number;
  };
  processingPlanMaterial?: {
    meta: Meta;
    id: string;
    name: string;
  };
  assortment: EntityRef;
  product: EntityRef;
  quantity: number;
  quantityPerProduct?: number;
}

export interface Demand {
  meta: Meta;
  id: string;
  name: string;
  externalCode?: string;
  moment: string;
  appied?: boolean;
  applicable: boolean;
  status: string;
  store: EntityRef;
  organization: EntityRef;
  agent: EntityRef;
  positions: DemandPosition[];
  created: string;
  updated: string;
}

export interface DemandPosition {
  id?: string;
  meta?: Meta;
  assortment: EntityRef;
  product: EntityRef;
  quantity: number;
  price: number;
  discount: number;
  vat: number;
  reserve?: number;
}

export interface WebhookEvent {
  meta: Meta;
  id: string;
  name: string;
  accountId: string;
  entityType: string;
  action: string;
  entity?: Demand;
  content?: {
    entity?: Demand;
    id?: string;
    type?: string;
  };
}

// Ответы API

export interface ApiResponse<T> {
  meta?: {
    href: string;
    type: string;
    mediaType: string;
    size: number;
    limit: number;
    offset: number;
  };
  rows?: T[];
  context?: {
    employee: {
      meta: Meta;
    };
  };
}

export interface StockAllResponse {
  meta: Meta;
  rows: StockRow[];
}

// Конфигурация приложения

export interface AppConfig {
  token: string;
  storeId: string;
  storeName: string;
  techCardFieldName: string;
  minStockThreshold: number;
}

// Логирование

export interface LogEntry {
  id: string;
  timestamp: string;
  type: 'info' | 'error' | 'warning' | 'success';
  message: string;
  details?: Record<string, unknown>;
}

// Результат обработки

export interface ProcessingResult {
  success: boolean;
  message: string;
  demandId?: string;
  demandName?: string;
  processingId?: string;
  processingName?: string;
  product?: {
    id: string;
    name: string;
    quantity: number;
    stockBefore: number;
  };
  error?: string;
}
